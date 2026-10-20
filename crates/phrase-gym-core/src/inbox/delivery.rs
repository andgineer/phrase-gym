use std::sync::Arc;

use phrase_gym_contracts::inbox::{
    JobFence, LessonOutput, LessonReceipt, LessonSubmission, PublicationReceipt,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::clock::Clock;
use crate::content::records::parse;
use crate::english::EnglishPackages;
use crate::error::CoreError;
use crate::hashing::canonical_json;
use crate::storage::Database;

use super::{input, jobs};

pub struct Delivery {
    database: Arc<Database>,
    validator: Arc<EnglishPackages>,
    clock: Arc<dyn Clock>,
}

fn stored(
    connection: &Connection,
    actor: &str,
    fence: &JobFence,
) -> Result<(Option<String>, Option<String>), CoreError> {
    connection.query_row("SELECT staged_output, publication FROM inbox_generation WHERE id = ?1 AND worker = ?2 AND claim_id = ?3 AND fence = ?4 AND token = ?5", params![fence.generation_id.to_string(), actor, fence.claim_id.to_string(), fence.fence, fence.token.to_string()], |row| Ok((row.get(0)?, row.get(1)?))).optional()?.ok_or_else(|| CoreError::Conflict("Unknown worker delivery; reconcile its claim".into()))
}

fn identity(connection: &Connection, output: &LessonOutput) -> Result<(), CoreError> {
    let previous: Option<(String, String)> = connection
        .query_row(
            "SELECT generation_id, request FROM inbox_delivery WHERE publication_id = ?1",
            [output.publication_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((generation, request)) = previous
        && (generation != output.fence.generation_id.to_string()
            || request != canonical_json(&(&output.adapter, &output.package)))
    {
        return Err(CoreError::Conflict(
            "Publication identity has different output or generation".into(),
        ));
    }
    Ok(())
}

fn binding(connection: &Connection, output: &LessonOutput) -> Result<(), CoreError> {
    let (request, receipt): (String, String) = connection.query_row(
        "SELECT request, receipt FROM inbox_generation WHERE id = ?1",
        [output.fence.generation_id.to_string()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let input: LessonSubmission = parse("input", &request)?;
    let receipt: LessonReceipt = parse("input receipt", &receipt)?;
    let package = &output.package;
    if package.namespace != receipt.namespace
        || package.source_revision != receipt.generation.to_string()
        || package.profile_revision != receipt.profile_revision
        || package.guide_version != receipt.guide_version
        || package.lesson.source_id != input.item_id
        || package.phrases.len() != input.phrases.len()
    {
        return Err(CoreError::Invalid(
            "Output must match its pinned input generation, lesson, profile and guide".into(),
        ));
    }
    for original in &input.phrases {
        let prepared = package
            .phrases
            .iter()
            .find(|p| p.source_id == original.source_id)
            .ok_or_else(|| {
                CoreError::Invalid("Output must retain every supplied phrase identity".into())
            })?;
        if prepared.text != original.text
            || original
                .reference_audio_id
                .is_some_and(|id| prepared.reference_audio_id != id)
            || prepared.expected_revision != original.expected_revision
        {
            return Err(CoreError::Invalid(
                "Output must retain exact supplied wording, original audio and expected revision"
                    .into(),
            ));
        }
        if original.reference_audio_id.is_none() {
            let media = crate::media::records::find(connection, prepared.reference_audio_id)?
                .ok_or_else(|| CoreError::Invalid("Generated reference is missing".into()))?;
            if media.metadata.origin != phrase_gym_contracts::media::MediaOrigin::Generated {
                return Err(CoreError::Invalid(
                    "Missing input audio requires generated reference provenance".into(),
                ));
            }
        }
    }
    input::expected_revisions(connection, &input, &receipt.namespace)
}

impl Delivery {
    pub fn new(
        database: Arc<Database>,
        validator: Arc<EnglishPackages>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            database,
            validator,
            clock,
        }
    }

    pub fn stage(&self, actor: &str, output: &LessonOutput) -> Result<(), CoreError> {
        let request = canonical_json(output);
        input::text(&output.adapter, "Adapter identity", 200)?;
        if output.publication_id.is_nil() || request.len() > 100_000 {
            return Err(CoreError::Invalid(
                "A stable publication identity and at most 100000 output bytes are required".into(),
            ));
        }
        // Committed identical deliveries remain recoverable after lease expiry or media loss.
        let committed = self.database.read(|tx| {
            identity(tx, output)?;
            let (old, receipt) = stored(tx, actor, &output.fence)?;
            if let Some(old) = old {
                if old != request {
                    return Err(CoreError::Conflict(
                        "Publication identity has different output".into(),
                    ));
                }
                return Ok(receipt.is_some());
            }
            Ok(false)
        })?;
        if committed {
            return Ok(());
        }
        self.validator.validate(&output.package)?;
        self.database.write(|tx| {
            jobs::check(tx, actor, &output.fence, self.clock.now())?;
            let (previous, _) = stored(tx, actor, &output.fence)?;
            if previous.is_some_and(|old| old != request) { return Err(CoreError::Conflict("Staged output is immutable".into())); }
            identity(tx, output)?;
            binding(tx, output)?;
            tx.execute("INSERT OR IGNORE INTO inbox_delivery VALUES (?1, ?2, ?3, ?4)", params![output.publication_id.to_string(), output.fence.generation_id.to_string(), canonical_json(&(&output.adapter, &output.package)), request])?;
            for id in crate::english::audio::references(&output.package) {
                tx.execute("INSERT OR IGNORE INTO inbox_output_media VALUES (?1, ?2)", params![output.fence.generation_id.to_string(), id.to_string()])?;
            }
            tx.execute("UPDATE inbox_generation SET status = 'pending_delivery', staged_output = ?2 WHERE id = ?1", params![output.fence.generation_id.to_string(), request])?;
            Ok(())
        })
    }

    pub fn receipt(
        &self,
        actor: &str,
        fence: &JobFence,
    ) -> Result<Option<PublicationReceipt>, CoreError> {
        self.database.read(|tx| {
            let (_, receipt) = stored(tx, actor, fence)?;
            receipt
                .map(|p| parse("publication receipt", &p))
                .transpose()
        })
    }

    pub fn publish(&self, actor: &str, fence: &JobFence) -> Result<PublicationReceipt, CoreError> {
        if let Some(receipt) = self.receipt(actor, fence)? {
            return Ok(receipt);
        }
        let output: LessonOutput = self.database.read(|tx| {
            let (output, _) = stored(tx, actor, fence)?;
            parse(
                "staged output",
                &output.ok_or_else(|| {
                    CoreError::Conflict("Stage the package before publication".into())
                })?,
            )
        })?;
        self.validator.validate(&output.package)?;
        self.database.write(|tx| {
            let (_, receipt) = stored(tx, actor, fence)?;
            if let Some(receipt) = receipt {
                return parse("publication receipt", &receipt);
            }
            let now = self.clock.now();
            jobs::check(tx, actor, fence, now)?;
            binding(tx, &output)?;
            let receipt = crate::english::publication::publish(
                tx,
                &output.package,
                fence.generation_id,
                output.publication_id,
                &now.to_rfc3339(),
            )?;
            tx.execute(
                "UPDATE inbox_generation SET status = 'published', publication = ?2 WHERE id = ?1",
                params![fence.generation_id.to_string(), canonical_json(&receipt)],
            )?;
            Ok(receipt)
        })
    }
}
