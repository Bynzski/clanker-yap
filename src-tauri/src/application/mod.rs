//! Application layer - use cases and shared application state.

mod state;
pub mod use_cases;
pub mod orchestrator;

pub use state::{AppState, RecordingState};