use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::readiness::CoachSummary;

use super::{Library, phrases};
use crate::error::CoreError;
use crate::media::index::MediaIndex;

impl Library {
    pub fn coach_summary(&self) -> Result<CoachSummary, CoreError> {
        let (targets, lessons) = self.database.read(|connection| {
            let targets = phrases::listed(connection, Language::En, None)?;
            let lessons = connection.query_row(
                "SELECT count(*) FROM lesson WHERE language = 'en'",
                [],
                |row| row.get(0),
            )?;
            Ok::<_, CoreError>((targets, lessons))
        })?;
        let media = MediaIndex::load(
            &self.database,
            &self.store,
            targets
                .iter()
                .flat_map(|target| phrases::media_ids(&target.content)),
        )?;
        let ready = targets
            .iter()
            .filter(|target| phrases::retrieval_ready(&target.content, &media, true))
            .count();
        let count = i64::try_from(targets.len()).unwrap_or(i64::MAX);
        let ready = i64::try_from(ready).unwrap_or(i64::MAX);
        let awaiting_audio = targets
            .iter()
            .filter(|target| {
                target.content.reference_audio_id.is_none()
                    || phrases::media_ids(&target.content).any(|id| !media.available(id))
            })
            .count();
        let awaiting_audio = i64::try_from(awaiting_audio).unwrap_or(i64::MAX);
        Ok(CoachSummary {
            lessons,
            targets: count,
            ready,
            study_only: count - ready - awaiting_audio,
            awaiting_audio,
        })
    }
}
