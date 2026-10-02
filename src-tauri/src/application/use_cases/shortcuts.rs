//! Select the desktop's shortcut backend and coordinate recording events.

use tauri::{AppHandle, Emitter};

use crate::application::{orchestrator, AppState};
use crate::domain::{AppError, Result};

pub fn managed_by_desktop() -> bool {
    #[cfg(target_os = "linux")]
    {
        crate::infrastructure::shortcuts::is_wayland()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

pub fn start(app: &AppHandle, state: &AppState) {
    let app = app.clone();
    let state = state.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = initialize(&app, &state).await {
            let message = error.to_string();
            tracing::error!(error = %message, "Shortcut setup failed");
            *state.last_error.lock() = Some(message.clone());
            let _ = app.emit(
                "transcription-error",
                serde_json::json!({ "error": message }),
            );
        }
    });
}

async fn initialize(app: &AppHandle, state: &AppState) -> Result<()> {
    let hotkey = state.settings.lock().hotkey.clone();
    #[cfg(target_os = "linux")]
    if managed_by_desktop() {
        use crate::infrastructure::shortcuts::{PortalSession, ShortcutEvent};
        let event_app = app.clone();
        let event_state = state.clone();
        let session = PortalSession::connect(&hotkey, move |event| match event {
            ShortcutEvent::Pressed => orchestrator::on_press(&event_app, &event_state),
            ShortcutEvent::Released => orchestrator::on_release(&event_app, &event_state),
            ShortcutEvent::Changed(label) => {
                *event_state.hotkey_display.lock() = label;
                let _ = event_app.emit("hotkey-changed", ());
            }
            ShortcutEvent::Unavailable(message) => {
                // Finish an active capture rather than leaving the microphone running.
                if matches!(
                    *event_state.recording.lock(),
                    crate::application::RecordingState::Recording { .. }
                ) {
                    orchestrator::on_release(&event_app, &event_state);
                }
                *event_state.last_error.lock() = Some(message.clone());
                let _ = event_app.emit(
                    "transcription-error",
                    serde_json::json!({ "error": message }),
                );
            }
        })
        .await?;
        *state.portal_shortcut.lock() = Some(session);
        tracing::info!(hotkey = %hotkey, "Native Wayland shortcut session registered");
        return Ok(());
    }
    if orchestrator::update_hotkey(app, state, &hotkey) {
        Ok(())
    } else {
        Err(AppError::SettingsInvalid(format!(
            "Could not register shortcut {hotkey}"
        )))
    }
}

pub async fn configure(state: &AppState) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let configuration = state
            .portal_shortcut
            .lock()
            .as_ref()
            .map(|session| session.configuration());
        if let Some((connection, path, version)) = configuration {
            return crate::infrastructure::shortcuts::configure(connection, path, version).await;
        }
    }
    let _ = state;
    Err(AppError::SettingsInvalid("Desktop shortcut setup is not ready. Check the app status and finish the desktop shortcut dialog.".into()))
}

/// The desktop's label is authoritative on Wayland; preserve the saved X11
/// preference independently so localized labels never get parsed as hotkeys.
pub fn display(state: &AppState, fallback: &str) -> String {
    state.hotkey_display.lock().clone().unwrap_or_else(|| {
        if managed_by_desktop() {
            "Set up desktop shortcut".into()
        } else {
            fallback.into()
        }
    })
}

pub fn validate_hotkey_update(hotkey: Option<&str>) -> Result<()> {
    if hotkey.is_some() && managed_by_desktop() {
        return Err(AppError::SettingsInvalid(
            "Configure this shortcut through the desktop shortcut dialog.".into(),
        ));
    }
    Ok(())
}
