//! Data transfer objects for the frontend API.

use serde::{Deserialize, Serialize};

/// Response for get_settings command.
#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub hotkey: String,
    pub model_path: String,
    pub model_name: String,
}

/// Request for update_settings command.
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub hotkey: Option<String>,
    pub model_path: Option<String>,
    pub model_name: Option<String>,
}

/// Response for update_settings command.
#[derive(Debug, Serialize)]
pub struct UpdateSettingsResponse {
    pub success: bool,
    pub message: String,
    pub requires_restart: bool,
}

/// A single transcription history item.
#[derive(Debug, Serialize)]
pub struct TranscriptionItem {
    pub id: String,
    pub text: String,
    pub duration_ms: i64,
    pub created_at: String,
}

/// Response for get_transcription_history command.
#[derive(Debug, Serialize)]
pub struct TranscriptionHistoryResponse {
    pub transcriptions: Vec<TranscriptionItem>,
}

/// Response for get_status command.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub state: String,
    pub last_error: Option<String>,
}