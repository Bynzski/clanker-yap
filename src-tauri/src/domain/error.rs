//! Unified error type for all application modules.

use serde::Serialize;
use thiserror::Error;

/// Application-wide error type. All fallible functions return `Result<T, AppError>`.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Whisper error: {0}")]
    Whisper(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Model not found at: {0}")]
    ModelNotFound(String),

    #[error("Microphone unavailable")]
    MicrophoneUnavailable,

    #[error("Paste injection failed: {0}")]
    PasteFailed(String),

    #[error("Settings invalid: {0}")]
    SettingsInvalid(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

/// AppError serializes to a string for Tauri command responses.
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn serializes_to_display_string() {
        let error = AppError::ModelNotFound("/tmp/model.bin".into());
        let serialized = serde_json::to_string(&error).unwrap();

        assert_eq!(serialized, "\"Model not found at: /tmp/model.bin\"");
    }
}
