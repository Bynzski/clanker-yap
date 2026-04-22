//! Cross-platform keystroke injection using enigo.

use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use enigo::{Enigo, Keyboard, Settings, Key, Direction};

use crate::domain::error::{AppError, Result};

/// Injects text into the focused input field via clipboard paste.
pub fn inject(app: &AppHandle, text: &str) -> Result<()> {
    // Write text to clipboard
    if let Err(e) = app.clipboard().write_text(text.to_string()) {
        return Err(AppError::PasteFailed(format!("clipboard: {}", e)));
    }

    // Simulate Ctrl+V / Cmd+V
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| AppError::PasteFailed(format!("enigo init: {}", e)))?;

    let paste_modifier = if cfg!(target_os = "macos") { Key::Meta } else { Key::Control };
    
    enigo.key(paste_modifier, Direction::Press)
        .map_err(|e| AppError::PasteFailed(format!("keystroke press: {}", e)))?;
    enigo.key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| AppError::PasteFailed(format!("keystroke click: {}", e)))?;
    enigo.key(paste_modifier, Direction::Release)
        .map_err(|e| AppError::PasteFailed(format!("keystroke release: {}", e)))?;

    Ok(())
}