use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::content::TargetInput;
use phrase_gym_contracts::library::{AudioStatus, Lesson, Phrase};
use rusqlite::Connection;
use uuid::Uuid;

use crate::content::{cues, records};
use crate::error::CoreError;
use crate::library::lessons;
use crate::library::search::Search;
use crate::media::index::MediaIndex;

pub(crate) struct Draft {
    pub id: Uuid,
    pub revision: i64,
    pub language: Language,
    pub content: TargetInput,
    pub lessons: Vec<Lesson>,
}

pub(crate) fn media_ids(content: &TargetInput) -> impl Iterator<Item = Uuid> + '_ {
    content
        .cues
        .iter()
        .filter_map(|cue| cue.audio_id)
        .chain(content.reference_audio_id)
}

pub(crate) fn retrieval_ready(content: &TargetInput, media: &MediaIndex) -> bool {
    let reference = content
        .reference_audio_id
        .is_some_and(|id| media.available(id));
    let cue_audio = content
        .cues
        .iter()
        .all(|cue| cue.audio_id.is_none_or(|id| media.available(id)));
    reference
        && cue_audio
        && content
            .cues
            .iter()
            .any(|cue| cues::has_visible_hint(content, cue))
}

impl Draft {
    pub(crate) fn finish(self, media: &MediaIndex) -> Phrase {
        let reference = self
            .content
            .reference_audio_id
            .is_some_and(|id| media.available(id));
        Phrase {
            retrieval_ready: retrieval_ready(&self.content, media),
            audio_status: if reference {
                AudioStatus::Ready
            } else {
                AudioStatus::Missing
            },
            id: self.id,
            revision: self.revision,
            language: self.language,
            content: self.content,
            lessons: self.lessons,
        }
    }
}

pub(crate) fn draft(
    connection: &Connection,
    id: Uuid,
    language: Language,
    revision: i64,
    content: TargetInput,
) -> Result<Draft, CoreError> {
    Ok(Draft {
        id,
        revision,
        language,
        content,
        lessons: lessons::of_target(connection, id)?,
    })
}

pub(crate) struct Listed {
    pub id: Uuid,
    pub revision: i64,
    pub content: TargetInput,
}

pub(crate) fn listed(
    connection: &Connection,
    language: Language,
    lesson: Option<Uuid>,
) -> Result<Vec<Listed>, CoreError> {
    let (sql, parameters) = match lesson {
        Some(lesson) => (
            "SELECT t.id, t.current_revision FROM target t \
             JOIN lesson_membership m ON m.target_id = t.id \
             WHERE t.language = ?1 AND m.lesson_id = ?2 ORDER BY m.position",
            vec![language.code().to_owned(), lesson.to_string()],
        ),
        None => (
            "SELECT id, current_revision FROM target WHERE language = ?1 ORDER BY id",
            vec![language.code().to_owned()],
        ),
    };
    let mut statement = connection.prepare(sql)?;
    let rows: Vec<(String, i64)> = statement
        .query_map(rusqlite::params_from_iter(parameters), |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|(id, revision)| {
            let id = records::uuid(&id)?;
            let content = records::content(connection, id, revision)?
                .ok_or_else(|| CoreError::NotFound("Content revision not found".to_owned()))?;
            Ok(Listed {
                id,
                revision,
                content,
            })
        })
        .collect()
}

pub(crate) fn matches(content: &TargetInput, search: &Search) -> bool {
    let haystack = std::iter::once(content.text.as_str())
        .chain(std::iter::once(content.purpose.as_str()))
        .chain(content.concepts.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");
    search.matches(&haystack)
}
