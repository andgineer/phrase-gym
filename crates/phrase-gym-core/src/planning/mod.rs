pub(crate) mod queue;
pub(crate) mod scope;

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use phrase_gym_contracts::library::{AudioStatus, Dialogue};
use phrase_gym_contracts::sessions::{
    Eligibility, ManifestSchema, PlannedTarget, PlannerVersion, SessionManifest, SessionStart,
};
use rusqlite::Connection;
use uuid::Uuid;

use crate::error::CoreError;
use crate::library::phrases;
use crate::media::index::MediaIndex;
use crate::prompts::selection;
use crate::scheduler::POLICY_VERSION;
use crate::schedules;

use scope::Scope;

const MAX_SESSION_TARGETS: usize = 20;
const MAX_NEW_TARGETS: usize = 5;

// Oldest due targets first, then at most five new targets in the remaining capacity; a dialogue
// session plans every prepared target of its script regardless of due time.
pub(crate) fn plan(
    connection: &Connection,
    start: &SessionStart,
    scope: Scope,
    now: DateTime<Utc>,
    media: &MediaIndex,
) -> Result<SessionManifest, CoreError> {
    let dialogue = scope.dialogue;
    let mut due: Vec<(DateTime<Utc>, PlannedTarget)> = Vec::new();
    let mut new = Vec::new();
    for candidate in scope.targets {
        let schedule = schedules::find(connection, candidate.id)?;
        let phrase = phrases::draft(
            connection,
            candidate.id,
            start.language,
            candidate.revision,
            candidate.content,
        )?
        .finish(media);
        let study = dialogue.is_none() && phrase.english.is_some();
        if !study && dialogue.is_none() && schedule.as_ref().is_some_and(|due| due.due_at > now) {
            continue;
        }
        if study && phrase.audio_status != AudioStatus::Ready {
            continue;
        }
        if !study && !phrase.retrieval_ready {
            continue;
        }
        let (cue_id, hidden_chunks) =
            if let Some(english) = phrase.english.as_ref().filter(|_| study) {
                let Some(situation) = english.situations.first() else {
                    continue;
                };
                (situation.id.clone(), Vec::new())
            } else {
                let Some(cue) =
                    selection::select(connection, phrase.id, phrase.revision, &phrase.content)?
                else {
                    continue;
                };
                (
                    cue.id.clone(),
                    selection::hidden_chunks(&phrase.content, &cue),
                )
            };
        let planned = PlannedTarget {
            hidden_chunks,
            cue_id,
            review_group_id: Uuid::new_v4(),
            schedule_version: schedule.as_ref().map_or(0, |due| due.version),
            eligibility: if study {
                Eligibility::Study
            } else if schedule.is_some() {
                Eligibility::Due
            } else {
                Eligibility::New
            },
            phrase,
        };
        match schedule {
            Some(schedule) => due.push((schedule.due_at, planned)),
            None => new.push(planned),
        }
    }
    due.sort_by_key(|(due_at, _)| *due_at);
    let mut targets: Vec<PlannedTarget> = due.into_iter().map(|(_, planned)| planned).collect();
    if dialogue.is_some() {
        targets.extend(new);
    } else {
        targets.truncate(MAX_SESSION_TARGETS);
        let budget = MAX_NEW_TARGETS.min(MAX_SESSION_TARGETS - targets.len());
        targets.extend(new.into_iter().take(budget));
    }
    let required_media_ids = required_media(&targets, dialogue.as_ref(), media)?;
    Ok(SessionManifest {
        schema_version: ManifestSchema,
        planner_version: PlannerVersion::DueNewV3Online,
        policy_version: POLICY_VERSION.to_owned(),
        language: start.language,
        lesson_id: start.lesson_id,
        targets,
        required_media_ids,
        dialogue,
    })
}

fn required_media(
    targets: &[PlannedTarget],
    dialogue: Option<&Dialogue>,
    media: &MediaIndex,
) -> Result<Vec<Uuid>, CoreError> {
    let mut ids = BTreeSet::new();
    for target in targets {
        let content = &target.phrase.content;
        ids.extend(content.reference_audio_id);
        ids.extend(
            content
                .cues
                .iter()
                .filter(|cue| cue.id == target.cue_id)
                .filter_map(|cue| cue.audio_id),
        );
    }
    for audio in dialogue.iter().flat_map(|dialogue| {
        dialogue
            .content
            .turns
            .iter()
            .filter_map(|turn| turn.audio_id)
    }) {
        if !media.available(audio) {
            return Err(CoreError::Conflict(
                "Dialogue partner audio is not prepared".to_owned(),
            ));
        }
        ids.insert(audio);
    }
    Ok(ids.into_iter().collect())
}
