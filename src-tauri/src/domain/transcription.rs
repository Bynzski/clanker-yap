//! Transcription entity with validation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::constants::{
    MAX_RECORDING_DURATION_MS, MAX_TRANSCRIPTION_LENGTH, MIN_RECORDING_DURATION_MS,
};

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

#[cfg(test)]
mod tests {
    use super::Transcription;
    use crate::domain::constants::{
        MAX_RECORDING_DURATION_MS, MAX_TRANSCRIPTION_LENGTH, MIN_RECORDING_DURATION_MS,
    };

    #[test]
    fn rejects_text_longer_than_max_length() {
        let text = "a".repeat(MAX_TRANSCRIPTION_LENGTH + 1);
        let error = Transcription::new(text, MIN_RECORDING_DURATION_MS).unwrap_err();

        assert!(error.to_string().contains("exceeds"));
    }

    #[test]
    fn rejects_duration_out_of_range() {
        let too_short =
            Transcription::new("hello".into(), MIN_RECORDING_DURATION_MS - 1).unwrap_err();
        assert!(too_short.to_string().contains("below minimum"));

        let too_long =
            Transcription::new("hello".into(), MAX_RECORDING_DURATION_MS + 1).unwrap_err();
        assert!(too_long.to_string().contains("exceeds maximum"));
    }
}
