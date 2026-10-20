use std::sync::Arc;

use chrono::{DateTime, Utc};
use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::readiness::DueSummary;
use uuid::Uuid;

use crate::clock::Clock;
use crate::error::CoreError;
use crate::library::{lessons, phrases};
use crate::media::index::MediaIndex;
use crate::media::store::MediaStore;
use crate::prompts::selection;
use crate::storage::{Database, StorageError};
use crate::{schedules, study, timestamps};

pub struct Readiness {
    database: Arc<Database>,
    store: Arc<MediaStore>,
    clock: Arc<dyn Clock>,
}

impl Readiness {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            database,
            store,
            clock,
        }
    }

    pub fn summary(
        &self,
        language: Language,
        lesson: Option<Uuid>,
    ) -> Result<DueSummary, CoreError> {
        let targets = self.database.read(|transaction| {
            if let Some(lesson) = lesson {
                lessons::require(transaction, lesson, language)?;
            }
            phrases::listed(transaction, language, lesson)?
                .into_iter()
                .map(|item| {
                    let schedule = schedules::find(transaction, item.id)?;
                    let english: bool = transaction.query_row(
                        "SELECT EXISTS(SELECT 1 FROM english_revision WHERE target_id = ?1 AND revision = ?2)",
                        rusqlite::params![item.id.to_string(), item.revision],
                        |row| row.get(0),
                    )?;
                    let cue = if english {
                        selection::situation(transaction, item.id, item.revision, &item.content)?
                    } else { selection::select(transaction, item.id, item.revision, &item.content)? };
                    let completed_study = english && study::completed(transaction, item.id, item.revision)?;
                    let needs_study = english && !completed_study;
                    let first_check = english && study::first_check(transaction, item.id, item.revision, schedule.as_ref())?;
                    Ok((item.content, schedule.map(|schedule| timestamps::isoformat(schedule.due_at)), cue.is_some(), english, needs_study, first_check))
                })
                .collect::<Result<Vec<_>, CoreError>>()
        })?;
        let ids: Vec<Uuid> = targets
            .iter()
            .flat_map(|(content, _, _, _, _, _)| phrases::media_ids(content))
            .collect();
        let media = MediaIndex::load(&self.database, &self.store, ids)?;
        let now = self.clock.now();
        let mut summary = DueSummary {
            due_count: 0,
            new_count: 0,
            preparing_count: 0,
            reference_only_count: 0,
            next_due_at: None,
        };
        let mut next: Option<DateTime<Utc>> = None;
        for (content, due, has_cue, english, needs_study, first_check) in targets {
            if !phrases::retrieval_ready(&content, &media, english) || !has_cue {
                summary.preparing_count += 1;
                continue;
            }
            if needs_study {
                summary.reference_only_count += 1;
                continue;
            }
            if first_check {
                summary.new_count += 1;
                continue;
            }
            let Some(due) = due else {
                summary.new_count += 1;
                continue;
            };
            let due = timestamps::parse(&due)
                .map_err(|error| StorageError::CorruptRecord(format!("due time {due}: {error}")))?;
            if due <= now {
                summary.due_count += 1;
            } else {
                next = Some(next.map_or(due, |earliest| earliest.min(due)));
            }
        }
        summary.next_due_at = next.map(timestamps::isoformat);
        Ok(summary)
    }
}
