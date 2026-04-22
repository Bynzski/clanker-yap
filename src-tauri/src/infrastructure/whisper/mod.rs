//! Whisper.cpp integration.

pub mod engine;
pub use engine::WhisperEngine;

use crate::application::AppState;
use crate::domain::error::Result;
use std::sync::Arc;

/// Lazily loads and caches the whisper engine.
pub fn load_or_get(state: &AppState) -> Result<Arc<WhisperEngine>> {
    let mut slot = state.whisper.lock();
    if let Some(engine) = slot.as_ref() {
        return Ok(engine.clone());
    }
    let path = state.settings.lock().model_path.clone();
    let engine = Arc::new(WhisperEngine::load(&path)?);
    *slot = Some(engine.clone());
    Ok(engine)
}
