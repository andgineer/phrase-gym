pub(crate) mod audio;
pub(crate) mod publication;
mod structure;
pub(crate) mod tasks;

use std::collections::BTreeMap;
use std::sync::Arc;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::english::PreparedEnglishLesson;
use uuid::Uuid;

use crate::content::records;
use crate::error::CoreError;
use crate::media::index::MediaIndex;
use crate::media::store::MediaStore;
use crate::profiles::Profiles;
use crate::storage::Database;

pub struct EnglishPackages {
    database: Arc<Database>,
    store: Arc<MediaStore>,
    profiles: Arc<Profiles>,
}

// Source identity is independent of wording, situation rotation, lesson title and item order.
pub fn target_id(namespace: &str, source_id: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        &crate::hashing::canonical_json(&("phrase-gym-english-target", namespace, source_id))
            .into_bytes(),
    )
}

impl EnglishPackages {
    pub fn new(database: Arc<Database>, store: Arc<MediaStore>, profiles: Arc<Profiles>) -> Self {
        Self {
            database,
            store,
            profiles,
        }
    }

    pub fn validate(
        &self,
        package: &PreparedEnglishLesson,
    ) -> Result<BTreeMap<String, Uuid>, CoreError> {
        structure::validate(package)?;
        let profile = self.profiles.get(Some(package.profile_revision))?.profile;
        let preferences = profile.languages.get(&Language::En).ok_or_else(|| {
            CoreError::Invalid("Pinned profile has no English preferences".into())
        })?;
        if preferences.card_language != Language::En
            || preferences.help_language.is_none()
            || preferences.translation_with_answer.is_none()
        {
            return Err(CoreError::Invalid(
                "English preview needs resolved English card/help preferences".into(),
            ));
        }
        for phrase in &package.phrases {
            if let Some(help) = &phrase.help {
                if Some(help.language) != preferences.help_language
                    || help.translation.is_some()
                        != (preferences.translation_with_answer == Some(true))
                {
                    return Err(CoreError::Invalid("Help language and answer translation must match the pinned personal profile".into()));
                }
            } else if preferences.translation_with_answer == Some(true) {
                return Err(CoreError::Invalid(
                    "Pinned profile requires translation with the answer".into(),
                ));
            }
        }
        let media = MediaIndex::load(&self.database, &self.store, audio::references(package))?;
        audio::validate(package, &media)?;
        self.database
            .read(|transaction| bindings(transaction, package))
    }
}

pub(crate) fn bindings(
    transaction: &rusqlite::Connection,
    package: &PreparedEnglishLesson,
) -> Result<BTreeMap<String, Uuid>, CoreError> {
    package
        .phrases
        .iter()
        .map(|phrase| {
            let existing =
                records::sourced_target(transaction, &package.namespace, &phrase.source_id)?;
            let id = if let Some(existing) = existing {
                if existing.language != "en"
                    || existing.kind != tasks::kind(phrase).code()
                    || existing.meaning_key != phrase.meaning_key
                {
                    return Err(CoreError::Invalid(
                        "Changed meaning requires a new target source identity".into(),
                    ));
                }
                if phrase.expected_revision != Some(existing.current_revision) {
                    return Err(CoreError::Conflict(
                        "Expected target revision changed".into(),
                    ));
                }
                existing.id
            } else {
                if phrase.expected_revision.is_some() {
                    return Err(CoreError::Conflict("Expected target does not exist".into()));
                }
                target_id(&package.namespace, &phrase.source_id)
            };
            Ok((phrase.source_id.clone(), id))
        })
        .collect()
}
