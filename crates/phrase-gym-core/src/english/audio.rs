use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::english::PreparedEnglishLesson;
use phrase_gym_contracts::media::MediaRole;
use uuid::Uuid;

use crate::error::CoreError;
use crate::media::index::MediaIndex;

pub(crate) fn references(package: &PreparedEnglishLesson) -> Vec<Uuid> {
    package
        .phrases
        .iter()
        .flat_map(|phrase| {
            std::iter::once(phrase.reference_audio_id)
                .chain(super::tasks::context_audio(phrase))
                .chain(phrase.situations.iter().flat_map(|situation| {
                    situation.audio_id.into_iter().chain(
                        situation
                            .partner
                            .as_ref()
                            .and_then(|partner| partner.audio_id),
                    )
                }))
        })
        .collect()
}

fn verified(index: &MediaIndex, id: Uuid, text: &str, role: MediaRole) -> Result<(), CoreError> {
    let record = index
        .record(id)
        .filter(|_| index.available(id))
        .ok_or_else(|| {
            CoreError::Invalid(format!(
                "Missing or damaged supplied audio {id}; upload the original clip"
            ))
        })?;
    let metadata = &record.metadata;
    if metadata.language != Language::En || metadata.text != text || metadata.role != role {
        return Err(CoreError::Invalid("Preview audio must be verified and match the exact English text and reference, cue or partner role".into()));
    }
    Ok(())
}

pub(super) fn validate(
    package: &PreparedEnglishLesson,
    index: &MediaIndex,
) -> Result<(), CoreError> {
    for phrase in &package.phrases {
        verified(
            index,
            phrase.reference_audio_id,
            &phrase.text,
            MediaRole::Reference,
        )?;
        if let Some(phrase_gym_contracts::english::EnglishTask::WordSense { contexts, .. }) =
            &phrase.task
        {
            for context in contexts {
                verified(
                    index,
                    context.reference_audio_id,
                    &context.answer,
                    MediaRole::Reference,
                )?;
            }
        }
        for situation in &phrase.situations {
            if let Some(id) = situation.audio_id {
                verified(index, id, &situation.text, MediaRole::Cue)?;
            }
            if let Some(partner) = &situation.partner
                && let Some(id) = partner.audio_id
            {
                verified(index, id, &partner.text, MediaRole::Partner)?;
            }
        }
    }
    Ok(())
}
