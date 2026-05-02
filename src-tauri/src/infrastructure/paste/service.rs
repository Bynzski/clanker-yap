//! Persistent paste controller — reuses a single Enigo instance.
//!
//! Creating a new `Enigo` on every paste triggers a fresh input-device session on
//! KDE/Wayland (via ei-portal), which causes repeated "Remote Control" permission
//! prompts. By initialising Enigo once and reusing it, we keep the same session
//! alive for the lifetime of the application — at most one permission prompt.

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::domain::error::{AppError, Result};

// ── PasteOutcome ────────────────────────────────────────────────────────────

/// Outcome of an inject call, used by the caller to decide what event to emit.
#[derive(Debug, Clone, PartialEq)]
pub enum PasteOutcome {
    /// Text was copied to the clipboard. The user must paste manually.
    CopiedOnly,
    /// Text was copied to the clipboard AND automatic keystroke paste was performed.
    CopiedAndPasted,
}

// ── PasteController ─────────────────────────────────────────────────────────

/// Owns a lazily-initialised `Enigo` instance and reuses it for every paste.
///
/// The Enigo is created on first use (which may trigger a single KDE permission
/// prompt on Wayland) and then held for the rest of the application session.
/// If initialisation fails (e.g. the user denies permission), the controller
/// records the failure and falls back to clipboard-only without retrying the
/// expensive init on every transcription.
pub struct PasteController {
    /// The persistent Enigo instance. `None` until first use.
    enigo: Option<Enigo>,
    /// When `true`, a previous Enigo init failed — don't keep retrying.
    init_failed: bool,
}

impl Default for PasteController {
    fn default() -> Self {
        Self::new()
    }
}

impl PasteController {
    /// Creates a new controller without initialising Enigo yet.
    pub fn new() -> Self {
        Self {
            enigo: None,
            init_failed: false,
        }
    }

    /// Returns a mutable reference to the Enigo instance, initialising it
    /// lazily on first call. If initialisation fails, records the failure
    /// and returns an error — subsequent calls will also fail immediately
    /// without re-attempting the expensive OS-level init.
    fn ensure_enigo(&mut self) -> Result<&mut Enigo> {
        if self.init_failed {
            return Err(AppError::PasteFailed(
                "Input controller unavailable (permission denied or init failed)".into(),
            ));
        }

        if self.enigo.is_none() {
            tracing::info!("Initialising persistent Enigo instance (first paste)");
            match Enigo::new(&Settings::default()) {
                Ok(e) => {
                    tracing::info!("Enigo instance created successfully");
                    self.enigo = Some(e);
                }
                Err(e) => {
                    tracing::error!(error = %e, "Enigo init failed — falling back to clipboard-only");
                    self.init_failed = true;
                    return Err(AppError::PasteFailed(format!(
                        "Input controller init failed: {}",
                        e
                    )));
                }
            }
        }

        Ok(self.enigo.as_mut().expect("enigo just initialised"))
    }

    /// Resets the controller, dropping the Enigo instance.
    /// Useful if the user re-enables auto-paste after a previous failure.
    pub fn reset(&mut self) {
        self.enigo = None;
        self.init_failed = false;
    }
}

// ── Public inject function ──────────────────────────────────────────────────

/// Copies `text` to the system clipboard. If `auto_paste` is `true`, also
/// simulates Ctrl+V / terminal paste keystrokes via the persistent Enigo
/// instance held by `controller`.
///
/// Returns `PasteOutcome::CopiedAndPasted` when keystroke injection ran,
/// `PasteOutcome::CopiedOnly` when only the clipboard was written.
pub fn inject(
    app: &AppHandle,
    text: &str,
    paste_mode: &str,
    auto_paste: bool,
    controller: &mut PasteController,
) -> Result<PasteOutcome> {
    // Always write text to clipboard first
    if let Err(e) = app.clipboard().write_text(text.to_string()) {
        return Err(AppError::PasteFailed(format!("clipboard: {}", e)));
    }

    if !auto_paste {
        tracing::info!("auto_paste disabled — text copied to clipboard only");
        return Ok(PasteOutcome::CopiedOnly);
    }

    // Attempt to get (or lazily init) the persistent Enigo instance
    let enigo = match controller.ensure_enigo() {
        Ok(e) => e,
        Err(e) => {
            // Init failed or previously failed — clipboard copy already succeeded,
            // so return CopiedOnly rather than propagating the error.
            tracing::warn!(error = ?e, "Enigo unavailable — text copied to clipboard only");
            return Ok(PasteOutcome::CopiedOnly);
        }
    };

    // Simulate keyboard paste
    if cfg!(any(target_os = "linux", target_os = "windows")) && paste_mode == "terminal" && send_terminal_paste(enigo).is_ok() {
        return Ok(PasteOutcome::CopiedAndPasted);
    }

    send_standard_paste(enigo)?;
    Ok(PasteOutcome::CopiedAndPasted)
}

// ── Keystroke helpers ───────────────────────────────────────────────────────

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
