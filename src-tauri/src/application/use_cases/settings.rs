//! Settings use cases.

use crate::domain::error::{AppError, Result};
use crate::domain::settings::Settings;
use crate::infrastructure::persistence::{Db, settings_repo};

/// Get the current application settings.
pub fn get_settings(db: &Db) -> Result<Settings> {
    settings_repo::load(db)
}

/// Update application settings.
pub fn update_settings(db: &Db, settings: Settings) -> Result<()> {
    settings_repo::save(db, &settings)
}