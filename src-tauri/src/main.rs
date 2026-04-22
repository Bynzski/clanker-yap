// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

use voice_transcribe_lib::{
    application::AppState,
    infrastructure::persistence::{db::Db, settings_repo},
};

fn main() {
    voice_transcribe_lib::init_logging();

    // Single instance enforcement - fail fast if another instance is running
    let _single_instance = tauri_plugin_single_instance::init(
        |app: &tauri::AppHandle<tauri::Wry>, _args, _cwd| {
            tracing::info!("Another instance attempted to start, focusing existing window");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        },
    );

    // Initialize database and settings
    let db = Db::open().expect("Failed to open database");
    let settings = settings_repo::load_or_init(&db).expect("Failed to load settings");

    tracing::info!(
        hotkey = %settings.hotkey,
        model_path = %settings.model_path,
        "Application starting"
    );

    let app_state = AppState::new(db, settings);

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            voice_transcribe_lib::presentation::commands::settings_cmds::get_settings,
            voice_transcribe_lib::presentation::commands::settings_cmds::update_settings,
            voice_transcribe_lib::presentation::commands::transcription_cmds::get_transcription_history,
            voice_transcribe_lib::presentation::commands::transcription_cmds::get_status,
        ])
        .setup(|_app| {
            tracing::info!("Tauri app setup complete");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                tracing::info!("Exit requested");
                // TODO: Phase 3 - clean up recorder and whisper engine
            }
        });
}