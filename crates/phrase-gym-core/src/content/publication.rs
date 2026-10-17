use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use phrase_gym_contracts::content::ContentPackage;
use phrase_gym_contracts::imports::{ChangeAction, ImportChange};
use rusqlite::{OptionalExtension, Transaction, params};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::content::records;
use crate::error::CoreError;
use crate::hashing::{canonical_json, fingerprint};
use crate::timestamps;

fn current_revision(transaction: &Transaction<'_>, target: Uuid) -> Result<i64, CoreError> {
    records::target(transaction, target)?
        .map(|row| row.current_revision)
        .ok_or_else(|| {
            CoreError::Conflict(format!("Target {target} disappeared during publication"))
        })
}

fn publish_targets(
    transaction: &Transaction<'_>,
    package: &ContentPackage,
    changes: &mut [ImportChange],
    now: &str,
) -> Result<BTreeMap<String, Uuid>, CoreError> {
    let mut target_ids = BTreeMap::new();
    for (content, change) in package.targets.iter().zip(changes.iter_mut()) {
        let target = change.target_id.unwrap_or_else(Uuid::new_v4);
        let revision = change.base_revision.unwrap_or(0) + 1;
        change.target_id = Some(target);
        target_ids.insert(content.source_id.clone(), target);
        let id = target.to_string();
        if change.action == ChangeAction::Create {
            transaction.execute(
                "INSERT INTO target (id, language, kind, meaning_key, recall_expression, \
                 current_revision) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    package.language.code(),
                    content.kind.code(),
                    content.meaning_key,
                    content.recall_expression,
                    revision
                ],
            )?;
            transaction.execute(
                "INSERT INTO source_item (namespace, source_id, target_id, source_revision) \
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    package.namespace,
                    content.source_id,
                    id,
                    package.source_revision
                ],
            )?;
        } else {
            transaction.execute(
                "UPDATE source_item SET source_revision = ?3 WHERE namespace = ?1 AND source_id = ?2",
                params![package.namespace, content.source_id, package.source_revision],
            )?;
        }
        if change.action == ChangeAction::Unchanged {
            continue;
        }
        transaction.execute(
            "UPDATE target SET current_revision = ?2 WHERE id = ?1",
            params![id, revision],
        )?;
        transaction.execute(
            "INSERT INTO target_revision (target_id, revision, payload, fingerprint, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                revision,
                canonical_json(content),
                fingerprint(content),
                now
            ],
        )?;
    }
    Ok(target_ids)
}

fn publish_lessons(
    transaction: &Transaction<'_>,
    package: &ContentPackage,
    target_ids: &BTreeMap<String, Uuid>,
) -> Result<(), CoreError> {
    for lesson in &package.lessons {
        let existing: Option<String> = transaction
            .query_row(
                "SELECT id FROM lesson WHERE namespace = ?1 AND source_id = ?2",
                params![package.namespace, lesson.source_id],
                |row| row.get(0),
            )
            .optional()?;
        let id = if let Some(id) = existing {
            transaction.execute(
                "UPDATE lesson SET title = ?2 WHERE id = ?1",
                params![id, lesson.title],
            )?;
            id
        } else {
            let id = Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO lesson (id, namespace, source_id, language, title) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    id,
                    package.namespace,
                    lesson.source_id,
                    package.language.code(),
                    lesson.title
                ],
            )?;
            id
        };
        transaction.execute("DELETE FROM lesson_membership WHERE lesson_id = ?1", [&id])?;
        for (position, source) in lesson.target_source_ids.iter().enumerate() {
            transaction.execute(
                "INSERT INTO lesson_membership (lesson_id, target_id, position) VALUES (?1, ?2, ?3)",
                params![id, target_ids[source].to_string(), i64::try_from(position).unwrap_or(i64::MAX)],
            )?;
        }
    }
    Ok(())
}

// A dialogue gets a new revision when its turns change or a linked target gains a revision, so
// every dialogue revision pins the exact target revisions it was published with.
fn publish_dialogues(
    transaction: &Transaction<'_>,
    package: &ContentPackage,
    target_ids: &BTreeMap<String, Uuid>,
) -> Result<(), CoreError> {
    for dialogue in &package.dialogues {
        let mut ids = BTreeMap::new();
        let mut revisions = BTreeMap::new();
        for source in dialogue
            .turns
            .iter()
            .filter_map(|turn| turn.target_source_id.as_ref())
        {
            let target = target_ids[source];
            ids.insert(source.clone(), target.to_string());
            revisions.insert(source.clone(), current_revision(transaction, target)?);
        }
        let bindings = json!({"ids": ids, "revisions": revisions});
        let payload = serde_json::to_value(dialogue).unwrap_or(Value::Null);
        let previous: Option<(String, i64, String, String)> = transaction
            .query_row(
                "SELECT d.id, d.current_revision, r.payload, r.bindings FROM dialogue d \
                 JOIN dialogue_revision r ON r.dialogue_id = d.id AND r.revision = d.current_revision \
                 WHERE d.namespace = ?1 AND d.source_id = ?2",
                params![package.namespace, dialogue.source_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        let (id, revision) = if let Some((id, revision, stored_payload, stored_bindings)) = previous
        {
            let same = records::parse::<Value>("dialogue payload", &stored_payload)? == payload
                && records::parse::<Value>("dialogue bindings", &stored_bindings)? == bindings;
            if same {
                continue;
            }
            transaction.execute(
                "UPDATE dialogue SET current_revision = ?2 WHERE id = ?1",
                params![id, revision + 1],
            )?;
            (id, revision + 1)
        } else {
            let id = Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO dialogue (id, namespace, source_id, language, current_revision) \
                     VALUES (?1, ?2, ?3, ?4, 1)",
                params![
                    id,
                    package.namespace,
                    dialogue.source_id,
                    package.language.code()
                ],
            )?;
            (id, 1)
        };
        transaction.execute(
            "INSERT INTO dialogue_revision (dialogue_id, revision, payload, bindings) \
             VALUES (?1, ?2, ?3, ?4)",
            params![
                id,
                revision,
                canonical_json(&payload),
                canonical_json(&bindings)
            ],
        )?;
    }
    Ok(())
}

// Runs inside the caller's write transaction, which also records the publication receipt.
pub fn publish(
    transaction: &Transaction<'_>,
    package: &ContentPackage,
    changes: &mut [ImportChange],
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    let now = timestamps::isoformat(now);
    let target_ids = publish_targets(transaction, package, changes, &now)?;
    publish_lessons(transaction, package, &target_ids)?;
    publish_dialogues(transaction, package, &target_ids)
}
