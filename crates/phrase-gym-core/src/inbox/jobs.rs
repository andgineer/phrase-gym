use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use phrase_gym_contracts::inbox::{JobClaim, JobFailure, JobFence};
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::clock::Clock;
use crate::content::records::parse;
use crate::error::CoreError;
use crate::hashing::canonical_json;
use crate::storage::Database;

pub const LEASE_SECONDS: i64 = 120;
pub const ATTEMPT_SECONDS: i64 = 600;
pub const MAX_ATTEMPTS: i64 = 3;
pub const DAILY_STARTS: i64 = 20;

pub struct Jobs {
    database: Arc<Database>,
    clock: Arc<dyn Clock>,
}

pub(super) fn check(
    connection: &Connection,
    actor: &str,
    fence: &JobFence,
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    let valid: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM inbox_generation g WHERE g.id = ?1 AND g.worker = ?2 AND g.claim_id = ?3 AND g.token = ?4 AND g.fence = ?5 AND g.status IN ('preparing', 'pending_delivery') AND julianday(g.lease_until) > julianday(?6) AND julianday(g.deadline) > julianday(?6) AND g.generation = (SELECT MAX(generation) FROM inbox_generation WHERE actor = g.actor AND item_id = g.item_id))",
        params![fence.generation_id.to_string(), actor, fence.claim_id.to_string(), fence.token.to_string(), fence.fence, now.to_rfc3339()], |row| row.get(0),
    )?;
    if !valid {
        return Err(CoreError::Conflict(
            "Worker lease expired, cancelled or superseded; reconcile the receipt before retrying"
                .into(),
        ));
    }
    Ok(())
}

impl Jobs {
    pub fn new(database: Arc<Database>, clock: Arc<dyn Clock>) -> Self {
        Self { database, clock }
    }

    pub fn claim(&self, actor: &str, claim_id: Uuid) -> Result<Option<JobClaim>, CoreError> {
        if claim_id.is_nil() {
            return Err(CoreError::Invalid(
                "A stable claim identity is required".into(),
            ));
        }
        let now = self.clock.now();
        self.database.write(|tx| {
            let previous: Option<(String, Option<String>)> = tx.query_row("SELECT actor, payload FROM preparation_claim WHERE claim_id = ?1", [claim_id.to_string()], |row| Ok((row.get(0)?, row.get(1)?))).optional()?;
            if let Some((owner, payload)) = previous {
                if owner != actor { return Err(CoreError::Conflict("Claim identity belongs to another worker".into())); }
                return payload.map(|p| parse("claim receipt", &p)).transpose();
            }
            tx.execute("UPDATE inbox_generation SET status = 'failed', reason = 'Preparation attempt limit reached', token = NULL WHERE status IN ('preparing', 'pending_delivery') AND julianday(lease_until) <= julianday(?1) AND attempts >= ?2", params![now.to_rfc3339(), MAX_ATTEMPTS])?;
            let starts: i64 = tx.query_row("SELECT COUNT(*) FROM preparation_claim WHERE payload IS NOT NULL AND substr(received_at, 1, 10) = ?1", [now.format("%Y-%m-%d").to_string()], |row| row.get(0))?;
            if starts >= DAILY_STARTS { return Err(CoreError::Conflict("Daily preparation start budget reached; retry on the next UTC day".into())); }
            let id: Option<String> = tx.query_row("SELECT id FROM inbox_generation WHERE (status = 'waiting' OR (status IN ('preparing', 'pending_delivery') AND julianday(lease_until) <= julianday(?1))) AND attempts < ?2 ORDER BY rowid LIMIT 1", params![now.to_rfc3339(), MAX_ATTEMPTS], |row| row.get(0)).optional()?;
            let claim = if let Some(id) = id {
                let (request, receipt, profile_revision, guide_version, fence): (String, String, i64, String, i64) = tx.query_row("SELECT request, receipt, profile_revision, guide_version, fence FROM inbox_generation WHERE id = ?1", [&id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)))?;
                let profile: String = tx.query_row("SELECT payload FROM profile_revision WHERE revision = ?1", [profile_revision], |row| row.get(0))?;
                let guide = tx.query_row("SELECT text FROM preparation_guide WHERE version = ?1", [guide_version], |row| row.get(0))?;
                let result = JobClaim { fence: JobFence { generation_id: crate::content::records::uuid(&id)?, claim_id, token: Uuid::new_v4(), fence: fence + 1 }, lease_until: (now + Duration::seconds(LEASE_SECONDS)).to_rfc3339(), deadline: (now + Duration::seconds(ATTEMPT_SECONDS)).to_rfc3339(), receipt: parse("input receipt", &receipt)?, input: parse("input", &request)?, profile: parse("profile", &profile)?, guide };
                tx.execute("UPDATE inbox_generation SET status = 'preparing', reason = NULL, attempts = attempts + 1, fence = ?2, worker = ?3, claim_id = ?4, token = ?5, lease_until = ?6, deadline = ?7, staged_output = NULL WHERE id = ?1", params![id, result.fence.fence, actor, claim_id.to_string(), result.fence.token.to_string(), result.lease_until, result.deadline])?;
                Some(result)
            } else { None };
            tx.execute("INSERT INTO preparation_claim VALUES (?1, ?2, ?3, ?4)", params![claim_id.to_string(), actor, now.to_rfc3339(), claim.as_ref().map(canonical_json)])?;
            Ok(claim)
        })
    }

    pub fn authorize_media(
        &self,
        actor: &str,
        fence: &JobFence,
        media_id: Uuid,
    ) -> Result<(), CoreError> {
        self.database.read(|tx| {
            check(tx, actor, fence, self.clock.now())?;
            let belongs: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM inbox_attachment WHERE generation_id = ?1 AND media_id = ?2)", params![fence.generation_id.to_string(), media_id.to_string()], |row| row.get(0))?;
            if !belongs { return Err(CoreError::NotFound("Media is not an attachment of this job".into())); }
            Ok(())
        })
    }

    pub fn renew(&self, actor: &str, fence: &JobFence) -> Result<String, CoreError> {
        let now = self.clock.now();
        self.database.write(|tx| {
            check(tx, actor, fence, now)?;
            let deadline: String = tx.query_row(
                "SELECT deadline FROM inbox_generation WHERE id = ?1",
                [fence.generation_id.to_string()],
                |row| row.get(0),
            )?;
            let deadline = crate::timestamps::parse(&deadline)
                .map_err(|error| CoreError::Invalid(error.to_string()))?;
            let until = (now + Duration::seconds(LEASE_SECONDS))
                .min(deadline)
                .to_rfc3339();
            tx.execute(
                "UPDATE inbox_generation SET lease_until = ?2 WHERE id = ?1",
                params![fence.generation_id.to_string(), until],
            )?;
            Ok(until)
        })
    }

    pub fn fail(&self, actor: &str, failure: &JobFailure) -> Result<(), CoreError> {
        super::input::text(&failure.reason, "Failure reason", 2_000)?;
        self.database.write(|tx| {
            check(tx, actor, &failure.fence, self.clock.now())?;
            tx.execute("UPDATE inbox_generation SET status = 'failed', reason = ?2, token = NULL WHERE id = ?1", params![failure.fence.generation_id.to_string(), failure.reason])?;
            Ok(())
        })
    }
}
