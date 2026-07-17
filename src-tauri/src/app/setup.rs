pub fn run() {
    crate::infrastructure::logging::init();

    let runtime = option_env!("APTABASE_KEY")
        .map(|_| tokio::runtime::Runtime::new().expect("failed to build Tokio runtime"));
    let _runtime_guard = runtime.as_ref().map(|rt| rt.enter());

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = crate::app::window::show(app);
        }))
        .plugin(tauri_plugin_autostart::Builder::new().args(["--hidden"]).build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());

    if let Some(key) = option_env!("APTABASE_KEY") {
        builder = builder.plugin(tauri_plugin_aptabase::Builder::new(key).build());
    }

    builder
        .setup(|app| {
            use tauri::Manager;

            let settings = crate::infrastructure::paths::load_settings(app.handle());
            app.manage(crate::state::app_state::AppState::new(settings));
            crate::app::tray::setup(app)?;
            if !std::env::args().any(|arg| arg == "--hidden") {
                let _ = crate::app::window::show(app.handle());
            }
            Ok(())
        })
        .on_window_event(crate::app::window::handle_event)
        .invoke_handler(tauri::generate_handler![
            crate::commands::ports::list_ports,
            crate::commands::ports::kill_process,
            crate::commands::ports::kill_process_elevated,
            crate::commands::ports::restart_process,
            crate::commands::ports::open_localhost_url,
            crate::commands::ports::copy_localhost_url,
            crate::commands::ports::copy_port,
            crate::commands::ports::copy_text,
            crate::commands::ports::open_path,
            crate::commands::ports::open_in_editor,
            crate::commands::ports::detect_vscode,
            crate::commands::settings::get_settings,
            crate::commands::settings::update_settings,
            crate::commands::window::show_popup_window,
            crate::commands::window::hide_popup_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PortPeek");
}
