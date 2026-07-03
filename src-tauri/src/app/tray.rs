use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App,
};

const OPEN_MENU_ID: &str = "open";
const QUIT_MENU_ID: &str = "quit";

pub fn setup(app: &mut App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, OPEN_MENU_ID, "Open PortPeek", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "Quit PortPeek", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("default window icon".into()))?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("PortPeek")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN_MENU_ID => {
                let result = crate::app::window::show(app);
                if result.is_ok() {
                    crate::app::analytics::track_tray_open(app, "menu");
                }
                log_error(result);
            }
            QUIT_MENU_ID => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let result = crate::app::window::toggle(tray.app_handle());
                if let Ok(true) = result {
                    crate::app::analytics::track_tray_open(tray.app_handle(), "left_click");
                }
                log_error(result.map(|_| ()));
            }
        })
        .build(app)?;

    Ok(())
}

fn log_error<T>(result: tauri::Result<T>) {
    if let Err(error) = result {
        tracing::error!(%error, "popup window operation failed");
    }
}
