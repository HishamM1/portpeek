mod app;
mod commands;
pub mod domain;
mod infrastructure;
pub mod platform;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app::setup::run();
}
