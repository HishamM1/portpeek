mod app;
mod commands;
mod domain;
mod infrastructure;
mod platform;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app::setup::run();
}
