//! Text injection use case.

use tauri::AppHandle;

use crate::application::AppState;
use crate::domain::error::Result;
use crate::infrastructure::paste;

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

    let mut controller = state.paste_controller.lock();
    paste::inject(app, text, &paste_mode, auto_paste, &mut controller)
}
