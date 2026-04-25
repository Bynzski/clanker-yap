//! Transcription use cases.

use crate::application::state::AppState;
use crate::domain::constants::MAX_HISTORY_ITEMS;
use crate::domain::transcription::{count_words, Transcription};
use crate::domain::Result;
use crate::infrastructure::persistence::{settings_repo, transcription_repo, Db};

/// Save a new transcription entry and update word count in settings.
pub fn save_transcription(db: &Db, transcription: &Transcription, state: &AppState) -> Result<()> {
    transcription_repo::save(db, transcription)?;
    transcription_repo::prune_to(db, MAX_HISTORY_ITEMS)?;

    // Increment total word count in settings
    let word_count = count_words(&transcription.text);
    if word_count > 0 {
        let mut settings = state.settings.lock();
        settings.total_words += word_count as u64;
        let updated_settings = settings.clone();
        drop(settings);
        settings_repo::save(db, &updated_settings)?;
    }

    Ok(())
}

/// Get recent transcription history.
pub fn get_history(db: &Db, limit: Option<u32>) -> Result<Vec<Transcription>> {
    transcription_repo::recent(db, limit.unwrap_or(MAX_HISTORY_ITEMS))
}
