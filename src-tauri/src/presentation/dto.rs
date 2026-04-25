//! Data transfer objects for the frontend API.

use serde::{Deserialize, Serialize};

/// Response for get_settings command.
#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub hotkey: String,
    pub model_path: String,
    pub model_name: String,
    pub paste_mode: String,
    pub audio_input: Option<crate::domain::settings::AudioInputSelection>,
    pub total_words: u64,
}

/// Information about the built-in model download option.
#[derive(Debug, Serialize)]
pub struct ModelDownloadInfoResponse {
    pub model_name: String,
    pub size_label: String,
    pub destination_path: String,
    pub source_url: String,
    pub installed: bool,
}

/// Request for update_settings command.
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub hotkey: Option<String>,
    pub model_path: Option<String>,
    pub model_name: Option<String>,
    pub paste_mode: Option<String>,
    pub audio_input: Option<crate::domain::settings::AudioInputSelection>,
}

/// Response for update_settings command.
#[derive(Debug, Serialize)]
pub struct UpdateSettingsResponse {
    pub success: bool,
    pub message: String,
    pub requires_restart: bool,
}

/// Response for download_default_model command.
#[derive(Debug, Serialize)]
pub struct DownloadModelResponse {
    pub success: bool,
    pub message: String,
    pub model_name: String,
    pub model_path: String,
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
