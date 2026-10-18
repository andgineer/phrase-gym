pub(crate) mod dialogues;
pub(crate) mod lessons;
pub(crate) mod phrases;
mod search;

use std::sync::Arc;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::library::{Dialogue, DialoguePage, LessonPage, Phrase, PhrasePage};
use uuid::Uuid;

use crate::content::records;
use crate::error::CoreError;
use crate::media::index::MediaIndex;
use crate::media::store::MediaStore;
use crate::storage::Database;

use phrases::Draft;
use search::Search;

#[derive(Clone, Copy, Debug)]
pub struct Paging {
    pub offset: i64,
    pub limit: i64,
}

impl Paging {
    fn window<T>(self, items: Vec<T>) -> Vec<T> {
        let skip = usize::try_from(self.offset).unwrap_or(usize::MAX);
        let take = usize::try_from(self.limit).unwrap_or(0);
        items.into_iter().skip(skip).take(take).collect()
    }
}

fn count(items: usize) -> i64 {
    i64::try_from(items).unwrap_or(i64::MAX)
}

pub struct Library {
    database: Arc<Database>,
    store: Arc<MediaStore>,
}

impl Library {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>) -> Self {
        Self { database, store }
    }

    fn finish(&self, drafts: Vec<Draft>) -> Result<Vec<Phrase>, CoreError> {
        let ids: Vec<Uuid> = drafts
            .iter()
            .flat_map(|draft| phrases::media_ids(&draft.content))
            .collect();
        let media = MediaIndex::load(&self.database, &self.store, ids)?;
        Ok(drafts
            .into_iter()
            .map(|draft| draft.finish(&media))
            .collect())
    }

    pub fn lessons(
        &self,
        language: Language,
        query: &str,
        paging: Paging,
    ) -> Result<LessonPage, CoreError> {
        let matches = self
            .database
            .read(|transaction| lessons::in_language(transaction, language, &Search::new(query)))?;
        Ok(LessonPage {
            total: count(matches.len()),
            items: paging.window(matches),
            offset: paging.offset,
            limit: paging.limit,
        })
    }

    pub fn phrases(
        &self,
        language: Language,
        query: &str,
        lesson: Option<Uuid>,
        paging: Paging,
    ) -> Result<PhrasePage, CoreError> {
        let search = Search::new(query.trim());
        let (drafts, total) = self.database.read(|transaction| {
            if let Some(lesson) = lesson {
                lessons::require(transaction, lesson, language)?;
            }
            let matching: Vec<_> = phrases::listed(transaction, language, lesson)?
                .into_iter()
                .filter(|item| phrases::matches(&item.content, &search))
                .collect();
            let total = count(matching.len());
            let drafts = paging
                .window(matching)
                .into_iter()
                .map(|item| {
                    phrases::draft(transaction, item.id, language, item.revision, item.content)
                })
                .collect::<Result<Vec<_>, CoreError>>()?;
            Ok::<_, CoreError>((drafts, total))
        })?;
        Ok(PhrasePage {
            items: self.finish(drafts)?,
            total,
            offset: paging.offset,
            limit: paging.limit,
        })
    }

    pub fn phrase(
        &self,
        id: Uuid,
        language: Language,
        revision: Option<i64>,
    ) -> Result<Phrase, CoreError> {
        let draft = self.database.read(|transaction| {
            let target = records::target(transaction, id)?
                .filter(|target| target.language == language.code())
                .ok_or_else(|| {
                    CoreError::NotFound("Phrase not found in this learning language".to_owned())
                })?;
            let revision = revision.unwrap_or(target.current_revision);
            let content = records::content(transaction, id, revision)?
                .ok_or_else(|| CoreError::NotFound("Content revision not found".to_owned()))?;
            phrases::draft(transaction, id, language, revision, content)
        })?;
        Ok(self.finish(vec![draft])?.remove(0))
    }

    pub fn dialogues(&self, language: Language, paging: Paging) -> Result<DialoguePage, CoreError> {
        let (items, total) = self.database.read(|transaction| {
            dialogues::page(transaction, language, paging.offset, paging.limit)
        })?;
        Ok(DialoguePage {
            items,
            total,
            offset: paging.offset,
            limit: paging.limit,
        })
    }

    pub fn dialogue(
        &self,
        id: Uuid,
        language: Language,
        revision: Option<i64>,
    ) -> Result<Dialogue, CoreError> {
        self.database
            .read(|transaction| dialogues::one(transaction, id, language, revision))
    }
}
