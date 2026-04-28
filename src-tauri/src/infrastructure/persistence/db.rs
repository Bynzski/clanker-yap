//! SQLite database connection wrapper.

use parking_lot::Mutex;
use rusqlite::Connection;

use crate::domain::error::{AppError, Result};

use super::paths::app_data_dir;

/// SQLite database wrapper. The connection is wrapped in an `Arc<Mutex<_>>`
/// so it can be cheaply cloned and sent across thread boundaries.
#[derive(Clone)]
pub struct Db {
    conn: std::sync::Arc<Mutex<Connection>>,
}

impl Db {
    /// Opens or creates the SQLite database.
    pub fn open() -> Result<Self> {
        let data_dir = app_data_dir()?;
        std::fs::create_dir_all(&data_dir).map_err(AppError::Io)?;

        let db_path = data_dir.join("voice-transcribe.db");
        let conn = Connection::open(&db_path).map_err(AppError::Sqlite)?;

        Self::from_connection(conn)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(AppError::Sqlite)?;
        Self::from_connection(conn)
    }

    fn from_connection(conn: Connection) -> Result<Self> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS app_settings (
                id          INTEGER PRIMARY KEY CHECK (id = 1),
                payload     TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS transcriptions (
                id          TEXT PRIMARY KEY,
                text        TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                created_at  TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_transcriptions_created_at
                ON transcriptions(created_at DESC);
            "#,
        )
        .map_err(AppError::Sqlite)?;

        Ok(Self {
            conn: std::sync::Arc::new(Mutex::new(conn)),
        })
    }

    /// Returns a clone of the inner Arc so blocking tasks get their own guard.
    pub fn clone_conn(&self) -> std::sync::Arc<Mutex<Connection>> {
        self.conn.clone()
    }

    /// Returns a reference to the connection guard.
    pub fn conn(&self) -> &std::sync::Arc<Mutex<Connection>> {
        &self.conn
    }
}
