//! Transcription use cases.

use crate::domain::constants::MAX_HISTORY_ITEMS;
use crate::domain::transcription::Transcription;
use crate::domain::Result;
use crate::infrastructure::persistence::{transcription_repo, Db};

/// Save a new transcription entry.
pub fn save_transcription(db: &Db, transcription: &Transcription) -> Result<()> {
    transcription_repo::save(db, transcription)?;
    transcription_repo::prune_to(db, MAX_HISTORY_ITEMS)?;
    Ok(())
}

/// Get recent transcription history.
pub fn get_history(db: &Db, limit: Option<u32>) -> Result<Vec<Transcription>> {
    transcription_repo::recent(db, limit.unwrap_or(MAX_HISTORY_ITEMS))
}
