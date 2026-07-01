use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager};

use crate::domain::settings::types::Settings;

const SETTINGS_FILE: &str = "settings.json";

pub fn load_settings(app: &AppHandle) -> Settings {
    let Ok(path) = settings_path(app) else {
        return Settings::default();
    };

    read_settings(&path)
        .or_else(|_| read_settings(&backup_path(&path)))
        .ok()
        .filter(|settings| settings.validate().is_ok())
        .unwrap_or_default()
}

pub fn save_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app).map_err(|error| error.to_string())?;
    let parent = path
        .parent()
        .ok_or_else(|| "settings path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    if read_settings(&path).is_ok() {
        fs::copy(&path, backup_path(&path)).map_err(|error| error.to_string())?;
    }

    let temporary = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
    let mut file = fs::File::create(&temporary).map_err(|error| error.to_string())?;
    file.write_all(&bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    fs::copy(&temporary, &path).map_err(|error| error.to_string())?;
    let _ = fs::remove_file(temporary);
    Ok(())
}

fn settings_path(app: &AppHandle) -> tauri::Result<PathBuf> {
    Ok(app.path().app_config_dir()?.join(SETTINGS_FILE))
}

fn read_settings(path: &Path) -> Result<Settings, io::Error> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}
