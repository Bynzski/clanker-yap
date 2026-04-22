//! Text injection use case.

use tauri::AppHandle;

use crate::application::AppState;
use crate::domain::error::Result;
use crate::infrastructure::paste;

/// Injects text via clipboard paste.
pub fn execute(app: &AppHandle, text: &str, state: &AppState) -> Result<()> {
    let paste_mode = state.settings.lock().paste_mode.clone();
    paste::inject(app, text, &paste_mode)
}
