//! Platform-specific application data directory handling.

use std::path::PathBuf;

use crate::domain::error::{AppError, Result};
use crate::domain::APP_DATA_SUBDIR;

/// Returns the application data directory path.
/// - macOS: ~/Library/Application Support/voice-transcribe/
/// - Linux: ~/.local/share/voice-transcribe/
/// - Windows: %APPDATA%\voice-transcribe\
pub fn app_data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|p| p.join(APP_DATA_SUBDIR))
        .ok_or_else(|| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine application data directory"
        )))
}