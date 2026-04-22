//! SQLite database connection wrapper.

use rusqlite::Connection;
use parking_lot::Mutex;
use std::path::PathBuf;

use crate::domain::error::{AppError, Result};
use super::paths::app_data_dir;

/// SQLite database wrapper.
pub struct Db {
    conn: parking_lot::Mutex<Connection>,
}

impl Db {
    /// Opens or creates the SQLite database.
    pub fn open() -> Result<Self> {
        let data_dir = app_data_dir()?;
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| AppError::Io(e))?;
        
        let db_path = data_dir.join("voice-transcribe.db");
        let conn = Connection::open(&db_path)
            .map_err(AppError::Sqlite)?;
        
        // Initialize schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS app_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                payload TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_transcriptions_created_at
                ON transcriptions(created_at DESC);"
        ).map_err(AppError::Sqlite)?;
        
        Ok(Self { conn: parking_lot::Mutex::new(conn) })
    }

    /// Returns a reference to the connection guard.
    pub fn conn(&self) -> &parking_lot::Mutex<Connection> {
        &self.conn
    }
}