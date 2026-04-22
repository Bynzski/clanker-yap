// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use voice_transcribe_lib::{
    application::{orchestrator, AppState},
    infrastructure::persistence::{db::Db, settings_repo},
};

fn main() {
    voice_transcribe_lib::init_logging();

    // Single instance enforcement — focus existing window if duplicate launch
    let single_instance =
        tauri_plugin_single_instance::init(|app: &tauri::AppHandle<tauri::Wry>, _args, _cwd| {
            tracing::info!("Another instance attempted to start, focusing existing window");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
            let _ = app.emit("app-already-running", ());
        });

    // Initialize database and settings
    let db = Db::open().expect("Failed to open database");
    let settings = settings_repo::load_or_init(&db).expect("Failed to load settings");

    let hotkey_str = settings.hotkey.clone();
    let model_path_str = settings.model_path.clone();
    tracing::info!(hotkey = %hotkey_str, model_path = %model_path_str, "Application starting");

    let app_state = AppState::new(db, settings);
    let app_state_for_setup = app_state.clone();
    let app_state_for_run = app_state.clone();

    tauri::Builder::default()
        .plugin(single_instance)
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            voice_transcribe_lib::presentation::commands::settings_cmds::get_settings,
            voice_transcribe_lib::presentation::commands::settings_cmds::update_settings,
            voice_transcribe_lib::presentation::commands::transcription_cmds::get_transcription_history,
            voice_transcribe_lib::presentation::commands::transcription_cmds::get_status,
        ])
        .setup(move |app| {
            tracing::info!("Tauri app setup starting");

            // Register global shortcut
            let shortcut: Shortcut = match hotkey_str.parse() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(error = ?e, "Invalid default hotkey '{}' — not registering", hotkey_str);
                    return Ok(());
                }
            };

            let app_handle = app.handle().clone();
            let state_handle: Arc<AppState> = Arc::new(app_state_for_setup);
            let state_for_error = state_handle.clone();
            let hotkey_for_error = hotkey_str.clone();
            let app_for_error = app.handle().clone();

            if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                match event.state {
                    ShortcutState::Pressed => orchestrator::on_press(&app_handle, &state_handle),
                    ShortcutState::Released => orchestrator::on_release(&app_handle, &state_handle),
                }
            }) {
                tracing::error!(error = ?e, "Failed to register global shortcut");
                *state_for_error.last_error.lock() =
                    Some(format!("Hotkey conflict: {} is already in use", hotkey_for_error));
                let _ = app_for_error.emit("hotkey-conflict", serde_json::json!({
                    "hotkey": hotkey_for_error
                }));
            } else {
                tracing::info!(hotkey = %hotkey_str, "Global shortcut registered");
            }

            // Validate model file exists on startup
            if !std::path::Path::new(&model_path_str).exists() {
                tracing::warn!(model_path = %model_path_str,
                    "Model file not found — transcription will fail until user provides one"
                );
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |_app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                tracing::info!("Exit requested — cleaning up");
                orchestrator::shutdown(&app_state_for_run);
            }
        });
}
