//! Transcription-related Tauri commands.

use tauri::State;

use crate::application::use_cases::transcription as transcription_usecase;
use crate::application::AppState;
use crate::domain::error::Result;
use crate::presentation::dto::{StatusResponse, TranscriptionHistoryResponse, TranscriptionItem};

/// Returns the recent transcription history.
#[tauri::command]
pub fn get_transcription_history(
    state: State<'_, AppState>,
) -> Result<TranscriptionHistoryResponse> {
    let transcriptions = transcription_usecase::get_history(&state.db, None)?;

    let items: Vec<TranscriptionItem> = transcriptions
        .into_iter()
        .map(|t| TranscriptionItem {
            id: t.id.to_string(),
            text: t.text,
            duration_ms: t.duration_ms,
            created_at: t.created_at.to_rfc3339(),
        })
        .collect();

    Ok(TranscriptionHistoryResponse {
        transcriptions: items,
    })
}

/// Returns the current recording/processing state.
#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> StatusResponse {
    let recording = state.recording.lock();
    let last_error = state.last_error.lock().clone();

    let state_str = match &*recording {
        crate::application::RecordingState::Idle => "Idle",
        crate::application::RecordingState::Recording { .. } => "Recording",
        crate::application::RecordingState::Processing => "Processing",
    };

    StatusResponse {
        state: state_str.to_string(),
        last_error,
    }
}
