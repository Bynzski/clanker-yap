//! Settings repository - persists settings as a single JSON row.

use chrono::Utc;
use rusqlite::params;

use crate::domain::error::{AppError, Result};
use crate::domain::settings::Settings;
use super::db::Db;

/// Loads settings from the database, or initializes with defaults if none exist.
pub fn load_or_init(db: &Db) -> Result<Settings> {
    let conn = db.conn().lock();
    
    let result: std::result::Result<String, rusqlite::Error> = conn.query_row(
        "SELECT payload FROM app_settings WHERE id = 1",
        [],
        |row| row.get(0),
    );
    
    match result {
        Ok(json) => serde_json::from_str(&json).map_err(AppError::Json),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let settings = Settings::default();
            save_internal(&conn, &settings)?;
            Ok(settings)
        }
        Err(e) => Err(AppError::Sqlite(e)),
    }
}

/// Loads settings from the database.
pub fn load(db: &Db) -> Result<Settings> {
    let conn = db.conn().lock();
    
    let json: String = conn.query_row(
        "SELECT payload FROM app_settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::SettingsInvalid("Settings not found".into())
        }
        _ => AppError::Sqlite(e),
    })?;
    
    serde_json::from_str(&json).map_err(AppError::Json)
}

/// Saves settings to the database.
pub fn save(db: &Db, settings: &Settings) -> Result<()> {
    let conn = db.conn().lock();
    save_internal(&conn, settings)
}

fn save_internal(conn: &rusqlite::Connection, settings: &Settings) -> Result<()> {
    let json = serde_json::to_string(settings).map_err(AppError::Json)?;
    let now = Utc::now().to_rfc3339();
    
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (id, payload, updated_at) VALUES (1, ?1, ?2)",
        params![json, now],
    ).map_err(AppError::Sqlite)?;
    
    Ok(())
}