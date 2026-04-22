//! Transcription use case wrapping whisper engine.

use std::sync::Arc;

use crate::application::AppState;
use crate::domain::error::Result;
use crate::infrastructure::whisper;

/// Transcribes audio samples using the whisper engine.
pub fn execute(samples: &[f32], state: &AppState) -> Result<String> {
    let engine = whisper::load_or_get(state)?;
    engine.transcribe(samples)
}