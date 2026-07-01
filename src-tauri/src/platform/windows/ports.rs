use std::{
    io,
    mem::{align_of, size_of},
    net::{Ipv4Addr, Ipv6Addr},
    ptr, slice,
};

use windows_sys::Win32::{
    Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR},
    NetworkManagement::IpHelper::{
        GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID,
        MIB_UDP6ROW_OWNER_PID, MIB_UDPROW_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
        UDP_TABLE_OWNER_PID,
    },
    Networking::WinSock::{AF_INET, AF_INET6},
};

use crate::domain::ports::types::{PortItem, PortProtocol};

pub fn list_tcp_listeners() -> io::Result<Vec<PortItem>> {
    let mut listeners = Vec::new();

    for row in table_rows::<MIB_TCPROW_OWNER_PID>(&query_table(Layer::Tcp, AF_INET as u32)?)? {
        listeners.push(port_item(
            Ipv4Addr::from(u32::from_be(row.dwLocalAddr)).to_string(),
            network_port(row.dwLocalPort),
            row.dwOwningPid,
            PortProtocol::Tcp,
        ));
    }

    for row in table_rows::<MIB_TCP6ROW_OWNER_PID>(&query_table(Layer::Tcp, AF_INET6 as u32)?)? {
        listeners.push(port_item(
            Ipv6Addr::from(row.ucLocalAddr).to_string(),
            network_port(row.dwLocalPort),
            row.dwOwningPid,
            PortProtocol::Tcp,
        ));
    }

    finish(listeners)
}

pub fn list_udp_listeners() -> io::Result<Vec<PortItem>> {
    let mut listeners = Vec::new();

    for row in table_rows::<MIB_UDPROW_OWNER_PID>(&query_table(Layer::Udp, AF_INET as u32)?)? {
        listeners.push(port_item(
            Ipv4Addr::from(u32::from_be(row.dwLocalAddr)).to_string(),
            network_port(row.dwLocalPort),
            row.dwOwningPid,
            PortProtocol::Udp,
        ));
    }

    for row in table_rows::<MIB_UDP6ROW_OWNER_PID>(&query_table(Layer::Udp, AF_INET6 as u32)?)? {
        listeners.push(port_item(
            Ipv6Addr::from(row.ucLocalAddr).to_string(),
            network_port(row.dwLocalPort),
            row.dwOwningPid,
            PortProtocol::Udp,
        ));
    }

    finish(listeners)
}

fn finish(mut listeners: Vec<PortItem>) -> io::Result<Vec<PortItem>> {
    listeners.sort_by(|left, right| {
        left.port
            .cmp(&right.port)
            .then_with(|| left.address.cmp(&right.address))
            .then_with(|| left.pid.cmp(&right.pid))
    });

    super::processes::enrich(&mut listeners);

    Ok(listeners)
}

#[derive(Clone, Copy)]
enum Layer {
    Tcp,
    Udp,
}

fn query_table(layer: Layer, address_family: u32) -> io::Result<Vec<u32>> {
    let mut byte_len = 0u32;
    let mut buffer = Vec::<u32>::new();

    loop {
        let data = if buffer.is_empty() {
            ptr::null_mut()
        } else {
            buffer.as_mut_ptr().cast()
        };

        // SAFETY: `data` is either null for the size query or points to a writable,
        // u32-aligned allocation of at least `byte_len` bytes. TCP and UDP owner-pid
        // tables share the same row-count-prefixed layout.
        let result = unsafe {
            match layer {
                Layer::Tcp => GetExtendedTcpTable(
                    data,
                    &mut byte_len,
                    1,
                    address_family,
                    TCP_TABLE_OWNER_PID_LISTENER,
                    0,
                ),
                Layer::Udp => GetExtendedUdpTable(
                    data,
                    &mut byte_len,
                    1,
                    address_family,
                    UDP_TABLE_OWNER_PID,
                    0,
                ),
            }
        };

        match result {
            NO_ERROR => return Ok(buffer),
            ERROR_INSUFFICIENT_BUFFER if byte_len > 0 => {
                buffer.resize((byte_len as usize).div_ceil(size_of::<u32>()), 0);
            }
            error => return Err(io::Error::from_raw_os_error(error as i32)),
        }
    }
}

fn table_rows<T>(buffer: &[u32]) -> io::Result<&[T]> {
    if buffer.is_empty() || align_of::<T>() > align_of::<u32>() {
        return Err(invalid_table());
    }

    let count = buffer[0] as usize;
    let required = count
        .checked_mul(size_of::<T>())
        .and_then(|rows| rows.checked_add(size_of::<u32>()))
        .ok_or_else(invalid_table)?;
    let available = buffer
        .len()
        .checked_mul(size_of::<u32>())
        .ok_or_else(invalid_table)?;

    if required > available {
        return Err(invalid_table());
    }

    // SAFETY: the length and alignment are checked above, and the table begins
    // immediately after the u32 row count according to the Windows API layout.
    let rows = unsafe {
        let first = buffer
            .as_ptr()
            .cast::<u8>()
            .add(size_of::<u32>())
            .cast::<T>();
        slice::from_raw_parts(first, count)
    };

    Ok(rows)
}

fn network_port(value: u32) -> u16 {
    u16::from_be(value as u16)
}

fn port_item(address: String, port: u16, pid: u32, protocol: PortProtocol) -> PortItem {
    let scheme = match protocol {
        PortProtocol::Tcp => "tcp",
        PortProtocol::Udp => "udp",
    };
    let url = matches!(protocol, PortProtocol::Tcp).then(|| format!("http://localhost:{port}"));

    PortItem {
        id: format!("{scheme}|{address}|{port}|{pid}"),
        port,
        address,
        protocol,
        pid: Some(pid),
        process_name: None,
        display_name: None,
        memory_mb: None,
        uptime_seconds: None,
        command: None,
        executable_path: None,
        working_directory: None,
        url,
        favicon_url: None,
        cached_favicon_path: None,
        framework: None,
        is_system_port: port < 1024,
    }
}

fn invalid_table() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid TCP table returned by Windows",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_validates_owner_pid_rows() {
        let buffer = vec![1, 5, 0x0100007f, 0x901f, 0, 0, 42];
        let rows = table_rows::<MIB_TCPROW_OWNER_PID>(&buffer).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].dwOwningPid, 42);
        assert!(table_rows::<MIB_TCPROW_OWNER_PID>(&[1, 0]).is_err());

        let socket = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = socket.local_addr().unwrap().port();
        let listeners = list_tcp_listeners().unwrap();
        assert!(listeners.iter().any(|listener| {
            listener.address == Ipv4Addr::LOCALHOST.to_string()
                && listener.port == port
                && listener.pid == Some(std::process::id())
        }));
    }

    #[test]
    fn lists_owned_udp_socket() {
        let socket = std::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = socket.local_addr().unwrap().port();
        let listeners = list_udp_listeners().unwrap();
        assert!(listeners.iter().any(|listener| {
            listener.protocol == PortProtocol::Udp
                && listener.port == port
                && listener.pid == Some(std::process::id())
        }));
    }
}
