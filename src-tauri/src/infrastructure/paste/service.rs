//! Cross-platform keystroke injection using enigo.

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::domain::error::{AppError, Result};

/// Injects text into the focused input field via clipboard paste.
pub fn inject(app: &AppHandle, text: &str, paste_mode: &str) -> Result<()> {
    // Write text to clipboard
    if let Err(e) = app.clipboard().write_text(text.to_string()) {
        return Err(AppError::PasteFailed(format!("clipboard: {}", e)));
    }

    // Simulate Ctrl+V / Cmd+V
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| AppError::PasteFailed(format!("enigo init: {}", e)))?;

    if cfg!(target_os = "linux")
        && paste_mode == "terminal"
        && send_terminal_paste(&mut enigo).is_ok()
    {
        return Ok(());
    }

    send_standard_paste(&mut enigo)?;

    Ok(())
}

fn send_standard_paste(enigo: &mut Enigo) -> Result<()> {
    let paste_modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };

    enigo
        .key(paste_modifier, Direction::Press)
        .map_err(|e| AppError::PasteFailed(format!("keystroke press: {}", e)))?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| AppError::PasteFailed(format!("keystroke click: {}", e)))?;
    enigo
        .key(paste_modifier, Direction::Release)
        .map_err(|e| AppError::PasteFailed(format!("keystroke release: {}", e)))?;

    Ok(())
}

fn send_terminal_paste(enigo: &mut Enigo) -> Result<()> {
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| AppError::PasteFailed(format!("terminal paste ctrl press: {}", e)))?;
    enigo
        .key(Key::Shift, Direction::Press)
        .map_err(|e| AppError::PasteFailed(format!("terminal paste shift press: {}", e)))?;
    let ctrl_shift_result = enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| AppError::PasteFailed(format!("terminal paste key click: {}", e)));
    let _ = enigo.key(Key::Shift, Direction::Release);
    let _ = enigo.key(Key::Control, Direction::Release);

    if ctrl_shift_result.is_ok() {
        return Ok(());
    }

    enigo
        .key(Key::Shift, Direction::Press)
        .map_err(|e| AppError::PasteFailed(format!("terminal fallback shift press: {}", e)))?;
    let shift_insert_result = enigo
        .key(Key::Insert, Direction::Click)
        .map_err(|e| AppError::PasteFailed(format!("terminal fallback insert click: {}", e)));
    let _ = enigo.key(Key::Shift, Direction::Release);

    shift_insert_result
}
