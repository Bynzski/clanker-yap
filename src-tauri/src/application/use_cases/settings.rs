//! Settings use cases.

use crate::domain::settings::Settings;
use crate::domain::Result;
use crate::infrastructure::persistence::{settings_repo, Db};

/// Get the current application settings.
pub fn get_settings(db: &Db) -> Result<Settings> {
    settings_repo::load(db)
}

/// Update application settings.
pub fn update_settings(db: &Db, settings: Settings) -> Result<()> {
    settings_repo::save(db, &settings)
}
