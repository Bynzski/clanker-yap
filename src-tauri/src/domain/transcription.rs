//! Transcription entity with validation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::constants::{MAX_TRANSCRIPTION_LENGTH, MAX_RECORDING_DURATION_MS, MIN_RECORDING_DURATION_MS};

/// A completed transcription entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transcription {
    /// Unique identifier (UUID v4).
    pub id: Uuid,

    /// Transcribed text content.
    pub text: String,

    /// Recording duration in milliseconds.
    pub duration_ms: i64,

    /// Timestamp of creation.
    pub created_at: DateTime<Utc>,
}

impl Transcription {
    /// Creates a new Transcription with validation.
    pub fn new(text: String, duration_ms: i64) -> crate::domain::Result<Self> {
        if text.len() > MAX_TRANSCRIPTION_LENGTH {
            return Err(crate::AppError::SettingsInvalid(format!(
                "Transcription text exceeds {} characters",
                MAX_TRANSCRIPTION_LENGTH
            )));
        }

        if duration_ms < MIN_RECORDING_DURATION_MS {
            return Err(crate::AppError::SettingsInvalid(format!(
                "Recording duration {}ms below minimum {}ms",
                duration_ms, MIN_RECORDING_DURATION_MS
            )));
        }

        if duration_ms > MAX_RECORDING_DURATION_MS {
            return Err(crate::AppError::SettingsInvalid(format!(
                "Recording duration {}ms exceeds maximum {}ms",
                duration_ms, MAX_RECORDING_DURATION_MS
            )));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            text,
            duration_ms,
            created_at: Utc::now(),
        })
    }
}