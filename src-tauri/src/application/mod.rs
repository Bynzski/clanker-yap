//! Application layer - use cases and shared application state.

pub mod orchestrator;
mod state;
pub mod use_cases;

pub use state::{AppState, RecordingState};
