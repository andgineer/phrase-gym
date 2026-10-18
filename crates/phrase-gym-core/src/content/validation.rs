use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::content::{ContentPackage, TargetInput};
use phrase_gym_contracts::imports::{ChangeAction, ImportChange, ImportReport, ImportStatus};
use phrase_gym_contracts::media::MediaRole;
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::content::{cues, records, version};
use crate::error::CoreError;
use crate::hashing::fingerprint;
use crate::media::index::MediaIndex;

pub fn referenced_media(package: &ContentPackage) -> Vec<Uuid> {
    let targets = package.targets.iter().flat_map(|target| {
        target
            .cues
            .iter()
            .filter_map(|cue| cue.audio_id)
            .chain(target.reference_audio_id)
    });
    let turns = package
        .dialogues
        .iter()
        .flat_map(|dialogue| dialogue.turns.iter().filter_map(|turn| turn.audio_id));
    targets.chain(turns).collect()
}

struct Audio<'a> {
    id: Option<Uuid>,
    language: Language,
    text: &'a str,
    role: MediaRole,
    context: &'a str,
}

struct Checker<'a> {
    connection: &'a Connection,
    package: &'a ContentPackage,
    media: &'a MediaIndex,
    errors: Vec<String>,
}

impl Checker<'_> {
    fn audio(&mut self, audio: &Audio<'_>) {
        let Some(id) = audio.id else {
            return;
        };
        let Some(record) = self.media.record(id).filter(|_| self.media.available(id)) else {
            self.errors.push(format!(
                "{}: missing reference media {id}; upload/recover it first",
                audio.context
            ));
            return;
        };
        let metadata = &record.metadata;
        if (metadata.language, metadata.text.as_str(), metadata.role)
            != (audio.language, audio.text, audio.role)
        {
            self.errors.push(format!(
                "{}: audio must match the exact language, text and role",
                audio.context
            ));
        }
    }

    fn target(&mut self, content: &TargetInput) -> Result<ImportChange, CoreError> {
        let package = self.package;
        let existing =
            records::sourced_target(self.connection, &package.namespace, &content.source_id)?;
        let mut action = ChangeAction::Create;
        if let Some(target) = &existing {
            let objective = (
                package.language.code(),
                content.kind.code(),
                content.meaning_key.as_str(),
                content.recall_expression.as_str(),
            );
            let stored = (
                target.language.as_str(),
                target.kind.as_str(),
                target.meaning_key.as_str(),
                target.recall_expression.as_str(),
            );
            if objective != stored {
                self.errors.push(format!(
                    "{}: changed learning objective; use a new source ID",
                    content.source_id
                ));
            }
            let current =
                records::revision_payload(self.connection, target.id, target.current_revision)?
                    .and_then(|payload| serde_json::from_str::<serde_json::Value>(&payload).ok());
            action = if current.is_some_and(|stored| fingerprint(&stored) == fingerprint(content)) {
                ChangeAction::Unchanged
            } else {
                ChangeAction::Update
            };
        }
        self.audio(&Audio {
            id: content.reference_audio_id,
            language: package.language,
            text: &content.text,
            role: MediaRole::Reference,
            context: &content.source_id,
        });
        for cue in &content.cues {
            let cue_context = format!("{}/{}", content.source_id, cue.id);
            if !cues::has_visible_hint(content, cue) {
                self.errors.push(format!(
                    "{cue_context}: retrieval cues need visible phrase-specific text; \
                     leave cues empty until a useful hint is prepared"
                ));
            }
            let spoken = cues::spoken_hint(content, cue);
            self.audio(&Audio {
                id: cue.audio_id,
                language: package.language,
                text: &spoken,
                role: MediaRole::Cue,
                context: &cue_context,
            });
        }
        Ok(ImportChange {
            source_id: content.source_id.clone(),
            action,
            target_id: existing.as_ref().map(|target| target.id),
            base_revision: existing.map(|target| target.current_revision),
        })
    }

    fn language_unchanged(
        &mut self,
        table: &str,
        kind: &str,
        source_id: &str,
    ) -> Result<(), CoreError> {
        let language: Option<String> = self
            .connection
            .query_row(
                &format!("SELECT language FROM {table} WHERE namespace = ?1 AND source_id = ?2"),
                params![self.package.namespace, source_id],
                |row| row.get(0),
            )
            .optional()?;
        if language.is_some_and(|language| language != self.package.language.code()) {
            self.errors.push(format!(
                "{kind} {source_id}: language changes require a new source ID"
            ));
        }
        Ok(())
    }
}

pub fn report(
    connection: &Connection,
    package: &ContentPackage,
    media: &MediaIndex,
) -> Result<ImportReport, CoreError> {
    let mut checker = Checker {
        connection,
        package,
        media,
        errors: Vec::new(),
    };
    let changes = package
        .targets
        .iter()
        .map(|content| checker.target(content))
        .collect::<Result<Vec<_>, _>>()?;
    for lesson in &package.lessons {
        checker.language_unchanged("lesson", "Lesson", &lesson.source_id)?;
    }
    for dialogue in &package.dialogues {
        checker.language_unchanged("dialogue", "Dialogue", &dialogue.source_id)?;
        for turn in &dialogue.turns {
            let role = if turn.target_source_id.is_some() {
                MediaRole::Reference
            } else {
                MediaRole::Partner
            };
            checker.audio(&Audio {
                id: turn.audio_id,
                language: package.language,
                text: &turn.text,
                role,
                context: &dialogue.source_id,
            });
        }
    }
    let errors = checker.errors;
    Ok(ImportReport {
        id: None,
        status: if errors.is_empty() {
            ImportStatus::Validated
        } else {
            ImportStatus::Rejected
        },
        changes,
        errors,
        base_content_version: version::content_version(connection, &package.namespace)?,
    })
}
