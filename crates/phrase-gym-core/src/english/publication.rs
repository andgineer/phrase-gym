use std::collections::BTreeMap;

use phrase_gym_contracts::content::{Chunk, Cue, Explanation, TargetInput};
use phrase_gym_contracts::english::{PreparedEnglishLesson, PreparedEnglishPhrase};
use phrase_gym_contracts::inbox::PublicationReceipt;
use rusqlite::{Transaction, params};
use uuid::Uuid;

use crate::error::CoreError;
use crate::hashing::{canonical_json, fingerprint};

fn projection(phrase: &PreparedEnglishPhrase) -> TargetInput {
    TargetInput {
        source_id: phrase.source_id.clone(),
        context_audio_ids: super::tasks::context_audio(phrase),
        kind: super::tasks::kind(phrase),
        meaning_key: phrase.meaning_key.clone(),
        recall_expression: phrase.intention_key.clone(),
        text: phrase.text.clone(),
        purpose: phrase.intention_key.clone(),
        chunks: vec![Chunk {
            id: "phrase".into(),
            text: phrase.text.clone(),
        }],
        required_chunks: vec!["phrase".into()],
        cues: phrase
            .situations
            .iter()
            .map(|s| Cue {
                id: s.id.clone(),
                prompt: s.text.clone(),
                visible_chunks: Vec::new(),
                reviewed: true,
                audio_id: s.audio_id,
            })
            .collect(),
        explanations: phrase
            .help
            .as_ref()
            .map(|h| {
                BTreeMap::from([(
                    h.language,
                    Explanation {
                        meaning: h.meaning.clone(),
                        usage: h.usage.clone(),
                        breakdown: Vec::new(),
                    },
                )])
            })
            .unwrap_or_default(),
        concepts: Vec::new(),
        acceptable_equivalents: phrase.accepted_variants.clone(),
        slots: super::tasks::slots(phrase),
        reference_audio_id: Some(phrase.reference_audio_id),
    }
}

pub(crate) fn publish(
    tx: &Transaction<'_>,
    package: &PreparedEnglishLesson,
    generation: Uuid,
    publication: Uuid,
    now: &str,
) -> Result<PublicationReceipt, CoreError> {
    let ids = super::bindings(tx, package)?;
    let lesson = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        canonical_json(&(
            "phrase-gym-lesson",
            &package.namespace,
            &package.lesson.source_id,
        ))
        .as_bytes(),
    );
    tx.execute("INSERT INTO lesson (id, namespace, source_id, language, title) VALUES (?1, ?2, ?3, 'en', ?4) ON CONFLICT (namespace, source_id) DO UPDATE SET title = excluded.title", params![lesson.to_string(), package.namespace, package.lesson.source_id, package.lesson.title])?;
    tx.execute(
        "DELETE FROM lesson_membership WHERE lesson_id = ?1",
        [lesson.to_string()],
    )?;
    for phrase in &package.phrases {
        let id = ids[&phrase.source_id];
        let revision = phrase.expected_revision.unwrap_or(0) + 1;
        let content = projection(phrase);
        tx.execute("INSERT INTO target (id, language, kind, meaning_key, recall_expression, current_revision) VALUES (?1, 'en', ?2, ?3, ?4, ?5) ON CONFLICT (id) DO UPDATE SET current_revision = excluded.current_revision", params![id.to_string(), content.kind.code(), phrase.meaning_key, phrase.intention_key, revision])?;
        tx.execute("INSERT INTO source_item (namespace, source_id, target_id, source_revision) VALUES (?1, ?2, ?3, ?4) ON CONFLICT (namespace, source_id) DO UPDATE SET source_revision = excluded.source_revision", params![package.namespace, phrase.source_id, id.to_string(), package.source_revision])?;
        tx.execute(
            "INSERT INTO target_revision VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id.to_string(),
                revision,
                canonical_json(&content),
                fingerprint(&content),
                now
            ],
        )?;
        tx.execute(
            "INSERT INTO english_revision VALUES (?1, ?2, ?3, ?4)",
            params![
                id.to_string(),
                revision,
                canonical_json(phrase),
                generation.to_string()
            ],
        )?;
    }
    let mut targets = Vec::new();
    for (position, source) in package.lesson.target_source_ids.iter().enumerate() {
        let id = ids[source];
        targets.push(id);
        tx.execute(
            "INSERT INTO lesson_membership VALUES (?1, ?2, ?3)",
            params![
                lesson.to_string(),
                id.to_string(),
                i64::try_from(position).unwrap_or(i64::MAX)
            ],
        )?;
    }
    Ok(PublicationReceipt {
        generation_id: generation,
        publication_id: publication,
        lesson_id: lesson,
        target_ids: targets,
        published_at: now.into(),
    })
}
