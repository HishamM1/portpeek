use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use crate::{domain::settings::types::Settings, infrastructure::paths, state::app_state::AppState};

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> Result<Settings, String> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| "settings state is unavailable".to_string())?
        .clone();
    let launch_at_startup = app
        .autolaunch()
        .is_enabled()
        .unwrap_or(settings.launch_at_startup);
    settings.launch_at_startup = launch_at_startup;
    state
        .settings
        .lock()
        .map_err(|_| "settings state is unavailable".to_string())?
        .launch_at_startup = launch_at_startup;
    Ok(settings)
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    settings.validate()?;
    let previous = state
        .settings
        .lock()
        .map_err(|_| "settings state is unavailable".to_string())?
        .clone();

    if settings.launch_at_startup != previous.launch_at_startup {
        set_autostart(&app, settings.launch_at_startup)?;
    }

    if let Err(error) = paths::save_settings(&app, &settings) {
        if settings.launch_at_startup != previous.launch_at_startup {
            let _ = set_autostart(&app, previous.launch_at_startup);
        }
        return Err(error);
    }

    *state
        .settings
        .lock()
        .map_err(|_| "settings state is unavailable".to_string())? = settings.clone();
    Ok(settings)
}

fn set_autostart(app: &AppHandle, enabled: bool) -> Result<(), String> {
    if enabled {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    }
    .map_err(|error| error.to_string())
}
