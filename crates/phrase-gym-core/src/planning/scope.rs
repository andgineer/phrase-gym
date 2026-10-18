use std::collections::{BTreeSet, HashMap};

use phrase_gym_contracts::content::TargetInput;
use phrase_gym_contracts::library::Dialogue;
use phrase_gym_contracts::sessions::SessionStart;
use rusqlite::Connection;
use uuid::Uuid;

use crate::content::records;
use crate::error::CoreError;
use crate::library::{dialogues, lessons, phrases};

pub(crate) struct Candidate {
    pub id: Uuid,
    pub revision: i64,
    pub content: TargetInput,
}

// Targets a start request may plan from, in stable ID order, with a dialogue's pinned revisions.
pub(crate) struct Scope {
    pub targets: Vec<Candidate>,
    pub dialogue: Option<Dialogue>,
}

impl Scope {
    pub fn media_ids(&self) -> BTreeSet<Uuid> {
        let turns = self.dialogue.iter().flat_map(|dialogue| {
            dialogue
                .content
                .turns
                .iter()
                .filter_map(|turn| turn.audio_id)
        });
        self.targets
            .iter()
            .flat_map(|candidate| phrases::media_ids(&candidate.content))
            .chain(turns)
            .collect()
    }
}

pub(crate) fn resolve(connection: &Connection, start: &SessionStart) -> Result<Scope, CoreError> {
    if start.lesson_id.is_some() && start.dialogue_id.is_some() {
        return Err(CoreError::Invalid(
            "Choose a lesson or a dialogue, not both".to_owned(),
        ));
    }
    if let Some(lesson) = start.lesson_id {
        lessons::require(connection, lesson, start.language)?;
    }
    let dialogue = start
        .dialogue_id
        .map(|id| dialogues::one(connection, id, start.language, None))
        .transpose()?;
    let pinned: HashMap<Uuid, i64> = dialogue
        .iter()
        .flat_map(|dialogue| {
            dialogue.target_ids.iter().filter_map(|(source, id)| {
                dialogue
                    .target_revisions
                    .get(source)
                    .map(|revision| (*id, *revision))
            })
        })
        .collect();
    let mut statement = connection.prepare(
        "SELECT id, current_revision FROM target WHERE language = ?1 \
         AND (?2 IS NULL OR id IN (SELECT target_id FROM lesson_membership WHERE lesson_id = ?2)) \
         ORDER BY id",
    )?;
    let rows: Vec<(String, i64)> = statement
        .query_map(
            rusqlite::params![
                start.language.code(),
                start.lesson_id.map(|id| id.to_string())
            ],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?
        .collect::<Result<_, _>>()?;
    let mut targets = Vec::new();
    for (id, current) in rows {
        let id = records::uuid(&id)?;
        if dialogue
            .as_ref()
            .is_some_and(|dialogue| !dialogue.target_ids.values().any(|bound| *bound == id))
        {
            continue;
        }
        let revision = pinned.get(&id).copied().unwrap_or(current);
        let content = records::content(connection, id, revision)?
            .ok_or_else(|| CoreError::NotFound("Content revision not found".to_owned()))?;
        targets.push(Candidate {
            id,
            revision,
            content,
        });
    }
    Ok(Scope { targets, dialogue })
}
