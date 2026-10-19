mod initial;
mod validation;

use std::sync::Arc;

use phrase_gym_contracts::profiles::{ProfileRestore, ProfileRevision, ProfileUpdate};
use rusqlite::{Connection, OptionalExtension, params};

use crate::clock::Clock;
use crate::content::records;
use crate::error::CoreError;
use crate::hashing::canonical_json;
use crate::storage::Database;

pub struct Profiles {
    database: Arc<Database>,
    clock: Arc<dyn Clock>,
}

fn read(connection: &Connection, revision: Option<i64>) -> Result<ProfileRevision, CoreError> {
    let payload: Option<String> = connection.query_row(
        "SELECT payload FROM profile_revision WHERE revision = COALESCE(?1, (SELECT MAX(revision) FROM profile_revision))",
        [revision], |row| row.get(0),
    ).optional()?;
    payload.map_or_else(
        || Err(CoreError::NotFound("Profile revision not found".into())),
        |payload| records::parse("profile revision", &payload),
    )
}

impl Profiles {
    pub fn new(database: Arc<Database>, clock: Arc<dyn Clock>) -> Self {
        Self { database, clock }
    }

    pub fn initialize_current_learner(&self) -> Result<(), CoreError> {
        let initial = ProfileRevision {
            revision: 1,
            profile: initial::current_learner(),
            mutation_id: None,
            actor: "initial-current-learner".into(),
            reason: "Specified personal initial values".into(),
            created_at: self.clock.now().to_rfc3339(),
            restored_from: None,
        };
        self.database.write(|transaction| transaction.execute(
            "INSERT OR IGNORE INTO profile_revision (revision, request, payload) VALUES (1, 'initial', ?1)",
            [canonical_json(&initial)],
        ))?;
        Ok(())
    }

    pub fn get(&self, revision: Option<i64>) -> Result<ProfileRevision, CoreError> {
        self.database
            .read(|transaction| read(transaction, revision))
    }

    pub fn update(
        &self,
        actor: &str,
        update: &ProfileUpdate,
    ) -> Result<ProfileRevision, CoreError> {
        self.mutate(
            actor,
            update.mutation_id,
            update.expected_revision,
            &update.reason,
            &canonical_json(update),
            None,
            Some(update.profile.clone()),
        )
    }

    pub fn restore(
        &self,
        actor: &str,
        restore: &ProfileRestore,
    ) -> Result<ProfileRevision, CoreError> {
        self.mutate(
            actor,
            restore.mutation_id,
            restore.expected_revision,
            &restore.reason,
            &canonical_json(restore),
            Some(restore.restore_revision),
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn mutate(
        &self,
        actor: &str,
        mutation: uuid::Uuid,
        expected: i64,
        reason: &str,
        request: &str,
        restore: Option<i64>,
        profile: Option<phrase_gym_contracts::profiles::PersonalProfile>,
    ) -> Result<ProfileRevision, CoreError> {
        if mutation.is_nil() || actor.is_empty() || reason.trim().is_empty() || reason.len() > 2_000
        {
            return Err(CoreError::Invalid(
                "A nonempty mutation identity and reason are required".into(),
            ));
        }
        let request = canonical_json(&(actor, request, restore));
        self.database.write(|transaction| {
            let existing: Option<(String, String)> = transaction.query_row(
                "SELECT request, payload FROM profile_revision WHERE mutation_id = ?1", [mutation.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            ).optional()?;
            if let Some((stored, payload)) = existing {
                return if stored == request { records::parse("profile receipt", &payload) } else { Err(CoreError::Conflict("Mutation identity has a different authorized request".into())) };
            }
            let current = read(transaction, None)?;
            if current.revision != expected {
                return Err(CoreError::Conflict("Profile revision changed; read the current profile before updating".into()));
            }
            let profile = match (profile, restore) {
                (Some(profile), None) => profile,
                (None, Some(revision)) => read(transaction, Some(revision))?.profile,
                _ => unreachable!("profile workflows select one source"),
            };
            validation::validate(&profile)?;
            let result = ProfileRevision { revision: current.revision + 1, profile, mutation_id: Some(mutation), actor: actor.into(), reason: reason.into(), created_at: self.clock.now().to_rfc3339(), restored_from: restore };
            transaction.execute("INSERT INTO profile_revision (revision, mutation_id, request, payload) VALUES (?1, ?2, ?3, ?4)", params![result.revision, mutation.to_string(), request, canonical_json(&result)])?;
            Ok(result)
        })
    }
}
