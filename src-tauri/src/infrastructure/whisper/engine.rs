//! Whisper.cpp engine wrapper.
//!
//! Audio contract: samples must be 16 kHz mono f32 in [-1, 1].

use std::sync::Arc;
use whisper_rs::{FullParams, SamplingStrategy};

use crate::domain::error::{AppError, Result};
use crate::domain::constants::MIN_RECORDING_DURATION_MS;

/// Wrapper around whisper-rs WhisperContext.
pub struct WhisperEngine {
    ctx: whisper_rs::WhisperContext,
}

impl WhisperEngine {
    /// Loads a GGML model from the given path.
    pub fn load(model_path: &str) -> Result<Self> {
        if !std::path::Path::new(model_path).exists() {
            return Err(AppError::ModelNotFound(model_path.into()));
        }
        
        let ctx = whisper_rs::WhisperContext::new_with_params(
            model_path,
            whisper_rs::WhisperContextParameters::default(),
        ).map_err(|e| AppError::Whisper(format!("load: {}", e)))?;
        
        Ok(Self { ctx })
    }

    /// Transcribes audio samples.
    /// 
    /// `samples` must be 16 kHz mono f32 in [-1, 1].
    /// Returns the transcribed text with leading/trailing whitespace trimmed.
    pub fn transcribe(&self, samples: &[f32]) -> Result<String> {
        // Duration validation
        let duration_ms = samples.len() as i64 * 1000 / 16_000;
        if duration_ms < MIN_RECORDING_DURATION_MS {
            return Err(AppError::Whisper("Audio too short".into()));
        }

        let mut state = self.ctx.create_state()
            .map_err(|e| AppError::Whisper(format!("create_state: {}", e)))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(num_cpus::get_physical() as i32);
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let t0 = std::time::Instant::now();
        state.full(params, samples)
            .map_err(|e| AppError::Whisper(format!("full: {}", e)))?;

        // Collect text from all segments using the iterator API
        let mut out = String::new();
        for segment in state.as_iter() {
            // to_str() returns Result<&str, WhisperError>
            if let Ok(text) = segment.to_str() {
                out.push_str(text);
            }
        }

        let elapsed_ms = t0.elapsed().as_millis() as u64;
        tracing::info!(samples = samples.len(), elapsed_ms, "transcribed");

        Ok(out.trim().to_string())
    }
}

use crate::application::AppState;

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