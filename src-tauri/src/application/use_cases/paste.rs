//! Text injection use case.

use tauri::AppHandle;

use crate::domain::error::Result;
use crate::infrastructure::paste;

/// Injects text via clipboard paste.
pub fn execute(app: &AppHandle, text: &str) -> Result<()> {
    paste::inject(app, text)
}
