use std::collections::HashMap;

use phrase_gym_contracts::sessions::{SessionManifest, SessionStep};
use uuid::Uuid;

use crate::error::CoreError;

// Dialogue sessions follow the script's learner turns, so a repeated target appears once per turn.
pub(crate) fn initial(manifest: &SessionManifest) -> Result<Vec<SessionStep>, CoreError> {
    let Some(dialogue) = &manifest.dialogue else {
        return Ok((0..manifest.targets.len()).map(SessionStep::new).collect());
    };
    let bindings: HashMap<Uuid, usize> = manifest
        .targets
        .iter()
        .enumerate()
        .map(|(index, target)| (target.phrase.id, index))
        .collect();
    let unprepared =
        || CoreError::Conflict("Every dialogue target needs prepared cues and audio".to_owned());
    let mut queue = Vec::new();
    for (turn_index, turn) in dialogue.content.turns.iter().enumerate() {
        let Some(source) = &turn.target_source_id else {
            continue;
        };
        let target = dialogue.target_ids.get(source).ok_or_else(unprepared)?;
        let index = *bindings.get(target).ok_or_else(unprepared)?;
        queue.push(SessionStep {
            dialogue_turn: Some(turn_index),
            ..SessionStep::new(index)
        });
    }
    Ok(queue)
}
