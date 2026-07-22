//! Clipboard copy and optional keyboard-simulated paste.

pub mod service;
pub use service::{copy_text, inject, PasteController, PasteOutcome};
