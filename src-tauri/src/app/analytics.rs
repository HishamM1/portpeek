use serde_json::json;
use tauri::{AppHandle, Manager};
use tauri_plugin_aptabase::EventTracker;

use crate::state::app_state::AppState;

pub fn track(app: &AppHandle, event: &str, props: Option<serde_json::Value>) {
    if option_env!("APTABASE_KEY").is_none() {
        return;
    }
    let enabled = app
        .state::<AppState>()
        .settings
        .lock()
        .map(|settings| settings.share_usage)
        .unwrap_or(false);
    if enabled {
        let _ = app.track_event(event, props);
    }
}

pub fn track_tray_open(app: &AppHandle, source: &str) {
    track(app, "tray_opened", Some(json!({ "source": source })));
}
