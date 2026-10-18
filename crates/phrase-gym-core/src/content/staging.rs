use std::sync::Arc;

use phrase_gym_contracts::content::ContentPackage;
use phrase_gym_contracts::imports::{ImportReport, ImportStatus};
use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use crate::clock::Clock;
use crate::content::{publication, records, structure, validation};
use crate::error::CoreError;
use crate::hashing::canonical_json;
use crate::media::index::MediaIndex;
use crate::media::store::MediaStore;
use crate::storage::Database;
use crate::timestamps;

pub struct PackageStaging {
    database: Arc<Database>,
    store: Arc<MediaStore>,
    clock: Arc<dyn Clock>,
}

impl PackageStaging {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            database,
            store,
            clock,
        }
    }

    fn media(&self, package: &ContentPackage) -> Result<MediaIndex, CoreError> {
        MediaIndex::load(
            &self.database,
            &self.store,
            validation::referenced_media(package),
        )
    }

    pub fn validate(&self, package: &ContentPackage) -> Result<ImportReport, CoreError> {
        structure::check(package)?;
        let media = self.media(package)?;
        self.database
            .read(|transaction| validation::report(transaction, package, &media))
    }

    pub fn stage(&self, package: &ContentPackage) -> Result<ImportReport, CoreError> {
        structure::check(package)?;
        let media = self.media(package)?;
        let staged_at = timestamps::isoformat(self.clock.now());
        self.database.write(|transaction| {
            let mut report = validation::report(transaction, package, &media)?;
            if !report.errors.is_empty() {
                return Ok(report);
            }
            let id = Uuid::new_v4();
            report.id = Some(id);
            report.status = ImportStatus::Staged;
            transaction.execute(
                "INSERT INTO staged_package (id, package, report, status, staged_at) \
                 VALUES (?1, ?2, ?3, 'staged', ?4)",
                params![
                    id.to_string(),
                    canonical_json(package),
                    canonical_json(&report),
                    staged_at
                ],
            )?;
            Ok(report)
        })
    }

    pub fn staged(&self, id: Uuid) -> Result<ImportReport, CoreError> {
        let report: Option<String> = self.database.read(|transaction| {
            transaction
                .query_row(
                    "SELECT report FROM staged_package WHERE id = ?1",
                    [id.to_string()],
                    |row| row.get(0),
                )
                .optional()
        })?;
        let report = report.ok_or_else(not_found)?;
        records::parse("staged package report", &report)
    }

    // A retry after commit returns the stored receipt instead of publishing twice.
    pub fn publish(&self, id: Uuid) -> Result<ImportReport, CoreError> {
        let package = self.staged_package(id)?;
        let media = self.media(&package)?;
        let now = self.clock.now();
        self.database.write(|transaction| {
            let (status, stored): (String, String) = transaction
                .query_row(
                    "SELECT status, report FROM staged_package WHERE id = ?1",
                    [id.to_string()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?
                .ok_or_else(not_found)?;
            let mut staged: ImportReport = records::parse("staged package report", &stored)?;
            if status == "published" {
                return Ok(staged);
            }
            let current = validation::report(transaction, &package, &media)?;
            if !current.errors.is_empty() {
                return Err(CoreError::Conflict(current.errors.join("; ")));
            }
            if current.changes != staged.changes
                || current.base_content_version != staged.base_content_version
            {
                return Err(CoreError::Conflict(
                    "Content changed after staging; validate and stage the package again"
                        .to_owned(),
                ));
            }
            publication::publish(transaction, &package, &mut staged.changes, now)?;
            staged.status = ImportStatus::Published;
            transaction.execute(
                "UPDATE staged_package SET status = 'published', report = ?2 WHERE id = ?1",
                params![id.to_string(), canonical_json(&staged)],
            )?;
            Ok(staged)
        })
    }

    fn staged_package(&self, id: Uuid) -> Result<ContentPackage, CoreError> {
        let package: Option<String> = self.database.read(|transaction| {
            transaction
                .query_row(
                    "SELECT package FROM staged_package WHERE id = ?1",
                    [id.to_string()],
                    |row| row.get(0),
                )
                .optional()
        })?;
        records::parse("staged package", &package.ok_or_else(not_found)?)
    }
}

fn not_found() -> CoreError {
    CoreError::NotFound("Import not found".to_owned())
}
