use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalRect, PhysicalSize, WebviewWindow, Window,
    WindowEvent,
};

const MAIN_WINDOW_LABEL: &str = "main";
const POPUP_GAP_LOGICAL_PX: f64 = 8.0;

pub fn handle_event(window: &Window, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }

    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.minimize();
    }
}

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    let window = main_window(app)?;
    position_default(&window)?;
    window.unminimize()?;
    window.show()?;
    window.set_focus()
}

pub fn hide(app: &AppHandle) -> tauri::Result<()> {
    main_window(app)?.hide()
}

pub fn minimize(app: &AppHandle) -> tauri::Result<()> {
    main_window(app)?.minimize()
}

pub fn toggle(app: &AppHandle) -> tauri::Result<bool> {
    let window = main_window(app)?;

    if window.is_visible()? && !window.is_minimized()? {
        window.minimize()?;
        return Ok(false);
    }

    position_default(&window)?;
    window.unminimize()?;
    window.show()?;
    window.set_focus()?;
    Ok(true)
}

fn main_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or(tauri::Error::WindowNotFound)
}

fn position_default(window: &WebviewWindow) -> tauri::Result<()> {
    let monitors = window.available_monitors()?;
    let Some(monitor) = monitors.first() else {
        return Ok(());
    };
    let scale = monitor.scale_factor();
    let gap = (POPUP_GAP_LOGICAL_PX * scale).round() as i32;
    let position = bottom_right(*monitor.work_area(), window.outer_size()?, gap);
    window.set_position(position)
}

fn bottom_right(
    work: PhysicalRect<i32, u32>,
    popup: PhysicalSize<u32>,
    gap: i32,
) -> PhysicalPosition<i32> {
    let right = work.position.x + work.size.width as i32;
    let bottom = work.position.y + work.size.height as i32;
    PhysicalPosition::new(
        (right - popup.width as i32 - gap).max(work.position.x + gap),
        (bottom - popup.height as i32 - gap).max(work.position.y + gap),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_right_sits_in_the_work_area_corner() {
        let position = bottom_right(
            PhysicalRect {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(1920, 1040),
            },
            PhysicalSize::new(420, 560),
            8,
        );
        assert_eq!(position, PhysicalPosition::new(1492, 472));
    }
}
