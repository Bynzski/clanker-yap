//! Transcription repository.

use rusqlite::params;
use chrono::Utc;

use crate::domain::error::{AppError, Result};
use crate::domain::transcription::Transcription;
use super::db::Db;

/// Saves a transcription entry.
pub fn save(db: &Db, transcription: &Transcription) -> Result<()> {
    let conn = db.conn().lock();
    
    conn.execute(
        "INSERT INTO transcriptions (id, text, duration_ms, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            transcription.id.to_string(),
            transcription.text,
            transcription.duration_ms,
            transcription.created_at.to_rfc3339(),
        ],
    ).map_err(AppError::Sqlite)?;
    
    Ok(())
}

/// Gets recent transcriptions ordered by created_at DESC.
pub fn recent(db: &Db, limit: u32) -> Result<Vec<Transcription>> {
    let conn = db.conn().lock();
    
    let mut stmt = conn.prepare(
        "SELECT id, text, duration_ms, created_at FROM transcriptions ORDER BY created_at DESC LIMIT ?1"
    ).map_err(AppError::Sqlite)?;
    
    let rows = stmt.query_map([limit], |row| {
        let id_str: String = row.get(0)?;
        let text: String = row.get(1)?;
        let duration_ms: i64 = row.get(2)?;
        let created_at_str: String = row.get(3)?;
        
        let id = uuid::Uuid::parse_str(&id_str).ok();
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .ok();
        
        Ok((id, text, duration_ms, created_at))
    }).map_err(AppError::Sqlite)?;
    
    let mut transcriptions = Vec::new();
    for row in rows {
        if let Some((Some(id), text, duration_ms, Some(created_at))) = row.ok().map(|(a, b, c, d)| (a, b, c, d)) {
            transcriptions.push(Transcription {
                id,
                text,
                duration_ms,
                created_at,
            });
        }
    }
    
    Ok(transcriptions)
}

/// Prunes transcriptions keeping only the newest `keep` items.
pub fn prune_to(db: &Db, keep: u32) -> Result<()> {
    let conn = db.conn().lock();
    
    conn.execute(
        "DELETE FROM transcriptions WHERE id NOT IN (
            SELECT id FROM transcriptions ORDER BY created_at DESC LIMIT ?1
        )",
        [keep],
    ).map_err(AppError::Sqlite)?;
    
    Ok(())
}