//! Text injection use case.

use tauri::AppHandle;

use crate::application::AppState;
use crate::domain::error::Result;
use crate::infrastructure::paste;
use crate::infrastructure::target_app;

/// Re-export the paste outcome type so callers don't need to reach into infrastructure.
pub use crate::infrastructure::paste::PasteOutcome;

/// Injects text via clipboard copy, optionally simulating keyboard paste.
///
/// Uses the persistent `PasteController` from `AppState` so that the Enigo
/// instance is created once and reused for the app's lifetime.
pub fn execute(app: &AppHandle, text: &str, state: &AppState) -> Result<PasteOutcome> {
    let settings = state.settings.lock();
    let paste_mode = settings.paste_mode.clone();
    let auto_paste = settings.auto_paste;
    drop(settings);

    let paste_mode = if paste_mode == "auto" {
        let paste_target_window = state.paste_target_window.lock().clone();
        let active_window = state.active_window.lock().clone();
        let window_for_classification = paste_target_window.as_ref().or(active_window.as_ref());
        let target = target_app::classify(window_for_classification);
        tracing::info!(
            effective_paste_mode = target.paste_mode(),
            resource_class = window_for_classification
                .map(|info| info.resource_class.as_str())
                .unwrap_or(""),
            desktop_file = window_for_classification
                .map(|info| info.desktop_file.as_str())
                .unwrap_or(""),
            caption = window_for_classification
                .map(|info| info.caption.as_str())
                .unwrap_or(""),
            "Resolved auto paste mode"
        );
        target.paste_mode().to_string()
    } else {
        paste_mode
    };

    let mut controller = state.paste_controller.lock();
    paste::inject(app, text, &paste_mode, auto_paste, &mut controller)
}
