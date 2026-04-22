//! Persistence infrastructure - SQLite storage and file paths.

pub mod db;
pub mod paths;
pub mod settings_repo;
pub mod transcription_repo;

pub use db::Db;
