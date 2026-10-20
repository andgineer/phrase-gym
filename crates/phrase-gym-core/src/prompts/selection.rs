use std::collections::{BTreeSet, HashMap};

use chrono::{DateTime, Utc};
use phrase_gym_contracts::content::{Cue, TargetInput};
use rusqlite::Connection;
use uuid::Uuid;

use crate::content::cues;
use crate::error::CoreError;

use super::history::{self, ChunkHistory};
use super::reports;

pub(crate) fn hidden_chunks(content: &TargetInput, cue: &Cue) -> Vec<String> {
    content
        .required_chunks
        .iter()
        .filter(|chunk| !cue.visible_chunks.contains(chunk))
        .cloned()
        .collect()
}

// The usable cue for a target revision: reviewed, with a phrase-specific hint, not revealing a word's
// required text and not reported for this revision.
pub(crate) fn select(
    connection: &Connection,
    target: Uuid,
    revision: i64,
    content: &TargetInput,
) -> Result<Option<Cue>, CoreError> {
    let reported = reports::reported_cues(connection, target, revision)?;
    let eligible: Vec<&Cue> = content
        .cues
        .iter()
        .filter(|cue| cues::practicable(content, cue) && !reported.contains(&cue.id))
        .collect();
    if eligible.is_empty() {
        return Ok(None);
    }
    let histories = history::load(connection, target)?;
    Ok(choose(content, &eligible, &histories).cloned())
}

pub(crate) fn situation(
    connection: &Connection,
    target: Uuid,
    revision: i64,
    content: &TargetInput,
) -> Result<Option<Cue>, CoreError> {
    let reported = reports::reported_cues(connection, target, revision)?;
    let mut statement = connection.prepare(
        "SELECT json_extract(payload, '$.cue_id'), COUNT(*), MAX(json_extract(payload, '$.presented_at')) \
         FROM practice_attempt WHERE target_id = ?1 AND revision = ?2 \
         AND json_extract(payload, '$.eligibility_reason') = 'eligible' \
         GROUP BY json_extract(payload, '$.cue_id')",
    )?;
    let histories = statement
        .query_map(rusqlite::params![target.to_string(), revision], |row| {
            Ok((
                row.get::<_, String>(0)?,
                (row.get::<_, i64>(1)?, row.get::<_, String>(2)?),
            ))
        })?
        .collect::<Result<HashMap<_, _>, _>>()?;
    Ok(content
        .cues
        .iter()
        .filter(|cue| cue.reviewed && !reported.contains(&cue.id))
        .min_by_key(|cue| histories.get(&cue.id).cloned().unwrap_or_default())
        .cloned())
}

type Priority<'a> = (i64, i64, Option<DateTime<Utc>>, &'a str);

// Among cues hiding as many required chunks as the first, prefer the least mastered and least
// presented hidden chunks, then the least recently presented, then the stable cue ID.
fn choose<'a>(
    content: &TargetInput,
    eligible: &[&'a Cue],
    histories: &HashMap<String, ChunkHistory>,
) -> Option<&'a Cue> {
    let hidden_count = |cue: &Cue| {
        let visible: BTreeSet<&String> = cue.visible_chunks.iter().collect();
        content
            .required_chunks
            .iter()
            .filter(|chunk| !visible.contains(chunk))
            .collect::<BTreeSet<_>>()
            .len()
    };
    let baseline = hidden_count(eligible.first()?);
    let priority = |cue: &'a Cue| -> Priority<'a> {
        let seen: Vec<&ChunkHistory> = hidden_chunks(content, cue)
            .iter()
            .filter_map(|chunk| histories.get(chunk))
            .collect();
        (
            seen.iter()
                .map(|history| history.recalls - history.failures)
                .sum(),
            seen.iter().map(|history| history.presentations).sum(),
            seen.iter()
                .filter_map(|history| history.last_presented)
                .max(),
            cue.id.as_str(),
        )
    };
    eligible
        .iter()
        .copied()
        .filter(|cue| hidden_count(cue) == baseline)
        .min_by(|left, right| priority(left).cmp(&priority(right)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> TargetInput {
        serde_json::from_value(serde_json::json!({
            "source_id": "day", "kind": "fixed_phrase", "meaning_key": "ask-day",
            "recall_expression": "How has your day been?", "text": "How has your day been?",
            "purpose": "Ask about their day.",
            "chunks": [{"id": "a", "text": "How has "}, {"id": "b", "text": "your day "},
                       {"id": "c", "text": "been?"}],
            "required_chunks": ["a", "b", "c"],
            "cues": [
                {"id": "z-hides-a", "prompt": "Ask.", "visible_chunks": ["b", "c"], "reviewed": true},
                {"id": "y-hides-b", "prompt": "Ask.", "visible_chunks": ["a", "c"], "reviewed": true},
                {"id": "x-hides-ab", "prompt": "Ask.", "visible_chunks": ["c"], "reviewed": true}
            ]
        }))
        .unwrap()
    }

    fn history(recalls: i64, failures: i64, presentations: i64) -> ChunkHistory {
        ChunkHistory {
            presentations,
            recalls,
            failures,
            last_presented: None,
        }
    }

    #[test]
    fn prefers_weaker_then_less_presented_chunks_among_comparable_cues() {
        let content = content();
        let eligible: Vec<&Cue> = content.cues.iter().collect();
        let fresh = HashMap::new();
        assert_eq!(choose(&content, &eligible, &fresh).unwrap().id, "y-hides-b");
        let weaker_a = HashMap::from([
            ("a".to_owned(), history(0, 2, 2)),
            ("b".to_owned(), history(2, 0, 2)),
        ]);
        assert_eq!(
            choose(&content, &eligible, &weaker_a).unwrap().id,
            "z-hides-a"
        );
        let less_presented_b = HashMap::from([
            ("a".to_owned(), history(1, 0, 3)),
            ("b".to_owned(), history(1, 0, 1)),
        ]);
        assert_eq!(
            choose(&content, &eligible, &less_presented_b).unwrap().id,
            "y-hides-b"
        );
        assert_eq!(
            hidden_chunks(&content, &content.cues[2]),
            vec!["a".to_owned(), "b".to_owned()]
        );
    }
}
