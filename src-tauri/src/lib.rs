//! Voice Transcription App - Local voice-to-text using whisper.cpp.
//!
//! Architecture:
//! - domain/     Pure types, constants, AppError
//! - application/ Use cases, AppState, orchestrator
//! - infrastructure/ whisper, audio, persistence, paste
//! - presentation/ Tauri commands and DTOs

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub use domain::{AppError, Result};

use tracing_subscriber::{fmt, EnvFilter};

/// Initializes tracing/logging once at startup.
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("voice_transcribe=info,warn"));

    fmt().with_env_filter(filter).with_target(false).init();
}
