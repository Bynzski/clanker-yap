//! Whisper.cpp engine wrapper.
//!
//! Audio contract: samples must be 16 kHz mono f32 in [-1, 1].

use whisper_rs::{FullParams, SamplingStrategy};

use crate::domain::constants::{
    MAX_WHISPER_INFERENCE_THREADS, MIN_RECORDING_DURATION_MS, WHISPER_SAMPLE_RATE,
};
use crate::domain::error::{AppError, Result};

fn inference_thread_count(physical_cores: usize) -> i32 {
    physical_cores.clamp(1, MAX_WHISPER_INFERENCE_THREADS) as i32
}

fn normalize_transcription_text(text: &str) -> String {
    let trimmed = text.trim();

    if trimmed.eq_ignore_ascii_case("[BLANK_AUDIO]") {
        String::new()
    } else {
        trimmed.to_string()
    }
}

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

        let load_started = std::time::Instant::now();
        let ctx = whisper_rs::WhisperContext::new_with_params(
            model_path,
            whisper_rs::WhisperContextParameters::default(),
        )
        .map_err(|e| AppError::Whisper(format!("load: {}", e)))?;

        tracing::info!(
            model_load_ms = load_started.elapsed().as_millis() as u64,
            "Whisper model loaded"
        );

        Ok(Self { ctx })
    }

    /// Transcribes audio samples.
    ///
    /// `samples` must be 16 kHz mono f32 in [-1, 1].
    /// Returns the transcribed text with leading/trailing whitespace trimmed.
    pub fn transcribe(&self, samples: &[f32]) -> Result<String> {
        // Duration validation
        let duration_ms = samples.len() as i64 * 1000 / WHISPER_SAMPLE_RATE as i64;
        if duration_ms < MIN_RECORDING_DURATION_MS {
            return Err(AppError::Whisper("Audio too short".into()));
        }

        let state_started = std::time::Instant::now();
        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| AppError::Whisper(format!("create_state: {}", e)))?;
        let state_create_ms = state_started.elapsed().as_millis() as u64;

        let physical_cores = num_cpus::get_physical();
        let inference_threads = inference_thread_count(physical_cores);
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(inference_threads);
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let t0 = std::time::Instant::now();
        state
            .full(params, samples)
            .map_err(|e| AppError::Whisper(format!("full: {}", e)))?;

        // Collect text from all segments using the iterator API.
        let mut out = String::new();
        for segment in state.as_iter() {
            if let Ok(text) = segment.to_str() {
                out.push_str(text);
            }
        }

        let elapsed_ms = t0.elapsed().as_millis() as u64;
        let realtime_factor = elapsed_ms as f64 / duration_ms as f64;
        tracing::info!(
            samples = samples.len(),
            audio_duration_ms = duration_ms,
            physical_cores,
            inference_threads,
            state_create_ms,
            elapsed_ms,
            realtime_factor,
            "transcribed"
        );

        Ok(normalize_transcription_text(&out))
    }
}

#[cfg(test)]
mod tests {
    use super::{inference_thread_count, normalize_transcription_text};
    use crate::domain::constants::MAX_WHISPER_INFERENCE_THREADS;

    #[test]
    fn inference_threads_preserve_smaller_cpu_counts() {
        assert_eq!(inference_thread_count(4), 4);
        assert_eq!(inference_thread_count(8), 8);
    }

    #[test]
    fn inference_threads_are_capped_for_large_and_hybrid_cpus() {
        assert_eq!(
            inference_thread_count(MAX_WHISPER_INFERENCE_THREADS + 6),
            MAX_WHISPER_INFERENCE_THREADS as i32
        );
    }

    #[test]
    fn inference_threads_never_return_zero() {
        assert_eq!(inference_thread_count(0), 1);
    }

    #[test]
    fn blank_audio_sentinel_is_normalized_to_empty_text() {
        assert_eq!(normalize_transcription_text(" [BLANK_AUDIO] "), "");
        assert_eq!(normalize_transcription_text("[blank_audio]"), "");
    }

    #[test]
    fn spoken_text_is_trimmed_and_preserved() {
        assert_eq!(
            normalize_transcription_text("  hello world  "),
            "hello world"
        );
    }
}
