use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalRect, PhysicalSize, Rect, WebviewWindow, Window,
    WindowEvent,
};

const MAIN_WINDOW_LABEL: &str = "main";
const POPUP_GAP_LOGICAL_PX: f64 = 8.0;

pub fn handle_event(window: &Window, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }

    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = window.hide();
        }
        WindowEvent::Focused(false) => {
            let _ = window.hide();
        }
        _ => {}
    }
}

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    let window = main_window(app)?;
    window.unminimize()?;
    window.show()?;
    window.set_focus()
}

pub fn hide(app: &AppHandle) -> tauri::Result<()> {
    main_window(app)?.hide()
}

pub fn toggle(app: &AppHandle, pointer: PhysicalPosition<f64>, tray: Rect) -> tauri::Result<()> {
    let window = main_window(app)?;

    if window.is_visible()? {
        return window.hide();
    }

    position_near_tray(&window, pointer, tray)?;
    window.unminimize()?;
    window.show()?;
    window.set_focus()
}

fn main_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or(tauri::Error::WindowNotFound)
}

fn position_near_tray(
    window: &WebviewWindow,
    pointer: PhysicalPosition<f64>,
    tray: Rect,
) -> tauri::Result<()> {
    let monitors = window.available_monitors()?;
    let monitor = monitors
        .iter()
        .find(|monitor| monitor_contains(monitor, pointer))
        .or_else(|| monitors.first());

    let Some(monitor) = monitor else {
        return Ok(());
    };

    let scale = monitor.scale_factor();
    let tray = PhysicalRect {
        position: tray.position.to_physical::<f64>(scale),
        size: tray.size.to_physical::<u32>(scale),
    };
    let gap = (POPUP_GAP_LOGICAL_PX * scale).round() as i32;
    let position = popup_position(tray, *monitor.work_area(), window.outer_size()?, gap);

    window.set_position(position)
}

fn monitor_contains(monitor: &tauri::Monitor, point: PhysicalPosition<f64>) -> bool {
    let position = monitor.position();
    let size = monitor.size();
    let right = position.x as f64 + size.width as f64;
    let bottom = position.y as f64 + size.height as f64;

    point.x >= position.x as f64
        && point.x < right
        && point.y >= position.y as f64
        && point.y < bottom
}

fn popup_position(
    tray: PhysicalRect<f64, u32>,
    work: PhysicalRect<i32, u32>,
    popup: PhysicalSize<u32>,
    gap: i32,
) -> PhysicalPosition<i32> {
    let left = work.position.x;
    let top = work.position.y;
    let right = left + work.size.width as i32;
    let bottom = top + work.size.height as i32;
    let tray_left = tray.position.x.round() as i32;
    let tray_top = tray.position.y.round() as i32;
    let tray_right = tray_left + tray.size.width as i32;
    let tray_bottom = tray_top + tray.size.height as i32;
    let center_x = tray_left + tray.size.width as i32 / 2;
    let center_y = tray_top + tray.size.height as i32 / 2;
    let popup_width = popup.width as i32;
    let popup_height = popup.height as i32;

    let nearest_edge = if center_y >= bottom {
        Edge::Bottom
    } else if center_y <= top {
        Edge::Top
    } else if center_x >= right {
        Edge::Right
    } else if center_x <= left {
        Edge::Left
    } else {
        nearest_edge(center_x, center_y, left, top, right, bottom)
    };

    let (x, y) = match nearest_edge {
        Edge::Bottom => (tray_right - popup_width, tray_top - popup_height - gap),
        Edge::Top => (tray_right - popup_width, tray_bottom + gap),
        Edge::Right => (tray_left - popup_width - gap, tray_bottom - popup_height),
        Edge::Left => (tray_right + gap, tray_bottom - popup_height),
    };

    PhysicalPosition::new(
        clamp_axis(x, left, right, popup_width, gap),
        clamp_axis(y, top, bottom, popup_height, gap),
    )
}

#[derive(Clone, Copy)]
enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

fn nearest_edge(x: i32, y: i32, left: i32, top: i32, right: i32, bottom: i32) -> Edge {
    let candidates = [
        ((y - top).abs(), Edge::Top),
        ((right - x).abs(), Edge::Right),
        ((bottom - y).abs(), Edge::Bottom),
        ((x - left).abs(), Edge::Left),
    ];

    candidates
        .into_iter()
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, edge)| edge)
        .unwrap_or(Edge::Bottom)
}

fn clamp_axis(value: i32, start: i32, end: i32, size: i32, gap: i32) -> i32 {
    let min = start + gap;
    let max = (end - size - gap).max(min);
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_popup_inside_horizontal_and_vertical_taskbars() {
        let popup = PhysicalSize::new(420, 560);

        let bottom = popup_position(
            PhysicalRect {
                position: PhysicalPosition::new(1880.0, 1040.0),
                size: PhysicalSize::new(40, 40),
            },
            PhysicalRect {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(1920, 1040),
            },
            popup,
            8,
        );
        assert_eq!(bottom, PhysicalPosition::new(1492, 472));

        let right = popup_position(
            PhysicalRect {
                position: PhysicalPosition::new(1880.0, 1040.0),
                size: PhysicalSize::new(40, 40),
            },
            PhysicalRect {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(1880, 1080),
            },
            popup,
            8,
        );
        assert_eq!(right, PhysicalPosition::new(1452, 512));
    }
}
