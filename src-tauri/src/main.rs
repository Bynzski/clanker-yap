// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Emitter, Manager};

#[cfg(any(target_os = "linux", target_os = "windows"))]
use voice_transcribe_lib::infrastructure::overlay;
#[cfg(target_os = "linux")]
use voice_transcribe_lib::infrastructure::target_app;
use voice_transcribe_lib::{
    application::{orchestrator, AppState},
    infrastructure::persistence::{db::Db, settings_repo},
};

fn main() {
    voice_transcribe_lib::init_logging();

    // Single instance enforcement — focus existing window if duplicate launch.
    // Note: We only focus the "main" window here. The "overlay" window should
    // never be focused — it's a click-through pill that should not interfere
    // with whatever application the user is typing in.
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

    let builder = tauri::Builder::default().plugin(single_instance);
    // The X11 plugin connects to an X server during setup, even without a
    // registered shortcut. Native Wayland shortcuts must not require XWayland.
    let builder = if voice_transcribe_lib::application::use_cases::shortcuts::managed_by_desktop() {
        builder
    } else {
        builder.plugin(tauri_plugin_global_shortcut::Builder::new().build())
    };

    builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            voice_transcribe_lib::presentation::commands::settings_cmds::get_settings,
            voice_transcribe_lib::presentation::commands::settings_cmds::configure_hotkey,
            voice_transcribe_lib::presentation::commands::settings_cmds::get_default_model_download_info,
            voice_transcribe_lib::presentation::commands::settings_cmds::update_settings,
            voice_transcribe_lib::presentation::commands::settings_cmds::download_default_model,
            voice_transcribe_lib::presentation::commands::transcription_cmds::get_transcription_history,
            voice_transcribe_lib::presentation::commands::transcription_cmds::copy_transcription_to_clipboard,
            voice_transcribe_lib::presentation::commands::transcription_cmds::get_status,
            voice_transcribe_lib::presentation::commands::audio_cmds::list_audio_inputs,
            voice_transcribe_lib::presentation::commands::window_cmds::sync_window_size,
            voice_transcribe_lib::presentation::commands::window_cmds::minimize_window,
            voice_transcribe_lib::presentation::commands::window_cmds::close_window,
            voice_transcribe_lib::presentation::commands::window_cmds::start_window_drag,
            voice_transcribe_lib::presentation::commands::window_cmds::get_app_version,
        ])
        .setup(move |app| {
            tracing::info!("Tauri app setup starting");

            #[cfg(target_os = "linux")]
            target_app::start_tracking(app_state_for_setup.active_window.clone());

            voice_transcribe_lib::application::use_cases::shortcuts::start(
                app.handle(), &app_state_for_setup,
            );

            // Validate model file exists on startup
            if !std::path::Path::new(&model_path_str).exists() {
                tracing::warn!(model_path = %model_path_str,
                    "Model file not found — transcription will fail until user provides one"
                );
            }

            // Create the overlay window, hidden by default.
            // This is the recording pill that appears during dictation.
            // Supported on Linux and Windows. macOS is excluded for now.
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            {
                if let Err(e) = overlay::create_overlay(app.handle()) {
                    tracing::error!(error = %e, "Failed to create overlay window — continuing without it");
                }
            }
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                tracing::debug!("Overlay window creation skipped — not supported on this platform");
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |app, event| {
            match event {
                tauri::RunEvent::WindowEvent { label, event, .. }
                    if label == "main"
                        && matches!(event, tauri::WindowEvent::CloseRequested { .. }) =>
                {
                    tracing::info!("Main window close requested — exiting app");
                    app.exit(0);
                }
                tauri::RunEvent::ExitRequested { .. } => {
                    tracing::info!("Exit requested — cleaning up");
                    // Pass AppHandle so shutdown can hide the overlay window
                    orchestrator::shutdown(app, &app_state_for_run);
                }
                _ => {}
            }
        });
}
