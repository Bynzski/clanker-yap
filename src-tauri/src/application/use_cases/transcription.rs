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

#[cfg(test)]
mod tests {
    use super::save_transcription;
    use crate::application::AppState;
    use crate::domain::settings::Settings;
    use crate::domain::transcription::Transcription;
    use crate::infrastructure::persistence::{db::Db, settings_repo};

    #[test]
    fn save_transcription_persists_cumulative_word_count() {
        let db = Db::open_in_memory().expect("in-memory db should open");
        let settings = settings_repo::load_or_init(&db).expect("settings should initialize");
        let state = AppState::new(db.clone(), settings);

        let first = Transcription::new("hello world".into(), 500).expect("valid transcription");
        save_transcription(&db, &first, &state).expect("first transcription should save");

        let second = Transcription::new("one two three".into(), 500).expect("valid transcription");
        save_transcription(&db, &second, &state).expect("second transcription should save");

        let persisted = settings_repo::load(&db).expect("settings should reload");

        assert_eq!(persisted.total_words, 5);
        assert_eq!(state.settings.lock().total_words, 5);
    }

    #[test]
    fn save_transcription_does_not_increment_count_for_empty_text() {
        let db = Db::open_in_memory().expect("in-memory db should open");
        let state = AppState::new(db.clone(), Settings::default());
        settings_repo::save(&db, &Settings::default()).expect("settings should save");

        let transcription = Transcription {
            text: String::new(),
            duration_ms: 500,
            ..Transcription::new("placeholder".into(), 500).expect("valid transcription")
        };

        save_transcription(&db, &transcription, &state).expect("transcription should save");

        let persisted = settings_repo::load(&db).expect("settings should reload");

        assert_eq!(persisted.total_words, 0);
        assert_eq!(state.settings.lock().total_words, 0);
    }
}
