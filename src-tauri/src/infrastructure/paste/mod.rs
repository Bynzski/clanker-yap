//! Clipboard copy and optional keyboard-simulated paste.

pub mod service;
pub use service::{inject, PasteController, PasteOutcome};
