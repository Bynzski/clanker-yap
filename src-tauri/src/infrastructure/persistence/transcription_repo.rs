//! Transcription repository.

use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use super::db::Db;
use crate::domain::error::{AppError, Result};
use crate::domain::transcription::Transcription;

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
    )
    .map_err(AppError::Sqlite)?;

    Ok(())
}

/// Gets recent transcriptions ordered by created_at DESC.
pub fn recent(db: &Db, limit: u32) -> Result<Vec<Transcription>> {
    let conn = db.conn().lock();

    let mut stmt = conn.prepare(
        "SELECT id, text, duration_ms, created_at FROM transcriptions ORDER BY created_at DESC LIMIT ?1"
    ).map_err(AppError::Sqlite)?;

    let rows = stmt
        .query_map([limit], |row| {
            let id_str: String = row.get(0)?;
            let text: String = row.get(1)?;
            let duration_ms: i64 = row.get(2)?;
            let created_at_str: String = row.get(3)?;

            let id = uuid::Uuid::parse_str(&id_str).ok();
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .ok();

            Ok((id, text, duration_ms, created_at))
        })
        .map_err(AppError::Sqlite)?;

    let mut transcriptions = Vec::new();
    for row in rows {
        if let Ok((Some(id), text, duration_ms, Some(created_at))) = row {
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

/// Finds a transcription by id.
pub fn find_by_id(db: &Db, id: Uuid) -> Result<Option<Transcription>> {
    let conn = db.conn().lock();

    let mut stmt = conn
        .prepare("SELECT id, text, duration_ms, created_at FROM transcriptions WHERE id = ?1")
        .map_err(AppError::Sqlite)?;

    let mut rows = stmt
        .query_map([id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let text: String = row.get(1)?;
            let duration_ms: i64 = row.get(2)?;
            let created_at_str: String = row.get(3)?;

            let parsed_id = Uuid::parse_str(&id_str).ok();
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .ok();

            Ok((parsed_id, text, duration_ms, created_at))
        })
        .map_err(AppError::Sqlite)?;

    let Some(row) = rows.next() else {
        return Ok(None);
    };

    match row {
        Ok((Some(id), text, duration_ms, Some(created_at))) => Ok(Some(Transcription {
            id,
            text,
            duration_ms,
            created_at,
        })),
        _ => Ok(None),
    }
}

/// Prunes transcriptions keeping only the newest `keep` items.
pub fn prune_to(db: &Db, keep: u32) -> Result<()> {
    let conn = db.conn().lock();

    conn.execute(
        "DELETE FROM transcriptions WHERE id NOT IN (
            SELECT id FROM transcriptions ORDER BY created_at DESC LIMIT ?1
        )",
        [keep],
    )
    .map_err(AppError::Sqlite)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{find_by_id, save};
    use crate::domain::transcription::Transcription;
    use crate::infrastructure::persistence::db::Db;

    #[test]
    fn find_by_id_returns_saved_transcription() {
        let db = Db::open_in_memory().expect("in-memory db should open");
        let transcription =
            Transcription::new("copy this full text".into(), 500).expect("valid transcription");

        save(&db, &transcription).expect("transcription should save");

        let found = find_by_id(&db, transcription.id)
            .expect("lookup should succeed")
            .expect("transcription should exist");

        assert_eq!(found.id, transcription.id);
        assert_eq!(found.text, transcription.text);
        assert_eq!(found.duration_ms, transcription.duration_ms);
    }

    #[test]
    fn find_by_id_returns_none_for_missing_transcription() {
        let db = Db::open_in_memory().expect("in-memory db should open");

        let found = find_by_id(&db, uuid::Uuid::new_v4()).expect("lookup should succeed");

        assert!(found.is_none());
    }
}
