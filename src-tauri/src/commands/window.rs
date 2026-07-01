use tauri::AppHandle;

#[tauri::command]
pub fn show_popup_window(app: AppHandle) -> Result<(), String> {
    crate::app::window::show(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn hide_popup_window(app: AppHandle) -> Result<(), String> {
    crate::app::window::hide(&app).map_err(|error| error.to_string())
}
