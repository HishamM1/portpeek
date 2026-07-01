use serde::{Deserialize, Serialize};

use crate::domain::detection::framework::FrameworkDetection;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PortItem {
    pub id: String,
    pub port: u16,
    pub address: String,
    pub protocol: PortProtocol,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub display_name: Option<String>,
    pub memory_mb: Option<f64>,
    pub uptime_seconds: Option<u64>,
    pub command: Option<String>,
    pub executable_path: Option<String>,
    pub working_directory: Option<String>,
    pub url: Option<String>,
    pub favicon_url: Option<String>,
    pub cached_favicon_path: Option<String>,
    pub framework: Option<FrameworkDetection>,
    pub is_system_port: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PortProtocol {
    Tcp,
    Udp,
}
