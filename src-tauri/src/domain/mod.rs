//! Domain layer - pure types, constants, and application error types.
//! No I/O, no external dependencies.

pub mod constants;
pub mod error;
pub mod settings;
pub mod transcription;

pub use constants::*;
pub use error::{AppError, Result};
pub use settings::Settings;
pub use transcription::Transcription;
