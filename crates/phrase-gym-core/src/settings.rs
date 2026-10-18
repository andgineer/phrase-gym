use std::sync::Arc;

use phrase_gym_contracts::settings::{Settings, SettingsUpdate};
use rusqlite::{OptionalExtension, params};

use crate::content::records;
use crate::error::CoreError;
use crate::hashing::canonical_json;
use crate::storage::Database;

// Device and interface settings are replaced as one complete set; personal preferences live apart.
pub struct DeviceSettings {
    database: Arc<Database>,
}

impl DeviceSettings {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn current(&self) -> Result<Settings, CoreError> {
        let stored: Option<String> = self.database.read(|transaction| {
            transaction
                .query_row(
                    "SELECT payload FROM device_settings WHERE id = 1",
                    [],
                    |row| row.get(0),
                )
                .optional()
        })?;
        stored.map_or(Ok(Settings::default()), |payload| {
            records::parse("device settings", &payload)
        })
    }

    pub fn replace(&self, update: &SettingsUpdate) -> Result<Settings, CoreError> {
        let settings = Settings::from(update.clone());
        self.database.write(|transaction| {
            transaction.execute(
                "INSERT INTO device_settings (id, payload) VALUES (1, ?1) \
                 ON CONFLICT (id) DO UPDATE SET payload = excluded.payload",
                params![canonical_json(&settings)],
            )
        })?;
        Ok(settings)
    }
}
