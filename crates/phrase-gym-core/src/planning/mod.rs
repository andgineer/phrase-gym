pub(crate) mod queue;
pub(crate) mod scope;

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use phrase_gym_contracts::library::Dialogue;
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
use crate::{schedules, study};

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
        let completed_study =
            phrase.english.is_some() && study::completed(connection, phrase.id, phrase.revision)?;
        let study = dialogue.is_none() && phrase.english.is_some() && !completed_study;
        let first_check = phrase.english.is_some()
            && study::first_check(connection, phrase.id, phrase.revision, schedule.as_ref())?;
        if !study
            && !first_check
            && dialogue.is_none()
            && !start.voluntary
            && schedule.as_ref().is_some_and(|due| due.due_at > now)
        {
            continue;
        }
        if !phrase.retrieval_ready {
            continue;
        }
        let cue = if phrase.english.is_some() {
            selection::situation(connection, phrase.id, phrase.revision, &phrase.content)?
        } else {
            selection::select(connection, phrase.id, phrase.revision, &phrase.content)?
        };
        let Some(cue) = cue else {
            continue;
        };
        let cue_id = cue.id.clone();
        let hidden_chunks = if study {
            Vec::new()
        } else {
            selection::hidden_chunks(&phrase.content, &cue)
        };
        let planned = PlannedTarget {
            hidden_chunks,
            cue_id,
            review_group_id: Uuid::new_v4(),
            schedule_version: schedule.as_ref().map_or(0, |due| due.version),
            eligibility: if study {
                Eligibility::Study
            } else if schedule.is_some() && !first_check {
                Eligibility::Due
            } else {
                Eligibility::New
            },
            phrase,
        };
        match schedule {
            Some(schedule) if !first_check => due.push((schedule.due_at, planned)),
            _ => new.push(planned),
        }
    }
    due.sort_by_key(|(due_at, _)| *due_at);
    let mut targets: Vec<PlannedTarget> = due.into_iter().map(|(_, planned)| planned).collect();
    if dialogue.is_some() || start.voluntary {
        targets.extend(new);
        if start.voluntary {
            targets.truncate(MAX_SESSION_TARGETS);
        }
    } else {
        targets.truncate(MAX_SESSION_TARGETS);
        let budget = MAX_NEW_TARGETS.min(MAX_SESSION_TARGETS - targets.len());
        targets.extend(new.into_iter().take(budget));
    }
    let required_media_ids = required_media(&targets, dialogue.as_ref(), media)?;
    Ok(SessionManifest {
        voluntary: start.voluntary,
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
