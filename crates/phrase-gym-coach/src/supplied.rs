use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::content::LessonInput;
use phrase_gym_contracts::english::{
    EnglishPackageVersion, PreparedEnglishLesson, PreparedEnglishPhrase,
};
use phrase_gym_contracts::inbox::{JobClaim, LessonOutput, LessonPhraseInput, LessonSubmission};
use phrase_gym_contracts::media::{
    MediaAsset, MediaMetadata, MediaOrigin, MediaRole, ReferenceMime,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuppliedLesson {
    pub lesson: LessonInput,
    pub phrase: PreparedEnglishPhrase,
    pub metadata: MediaMetadata,
    pub mime_type: ReferenceMime,
    pub assessment_version: String,
}

impl SuppliedLesson {
    pub fn parse(json: &str) -> Result<Self, String> {
        if json.len() > 8_000 {
            return Err("Prepared lesson JSON must be at most 8000 bytes".into());
        }
        let supplied: Self = serde_json::from_str(json)
            .map_err(|_| "Supply the complete prepared single-phrase lesson JSON".to_owned())?;
        if supplied.metadata.language != Language::En
            || supplied.metadata.text != supplied.phrase.text
            || supplied.metadata.role != MediaRole::Reference
            || supplied.metadata.origin == MediaOrigin::Generated
            || supplied.phrase.expected_revision.is_some()
            || supplied.lesson.target_source_ids != [supplied.phrase.source_id.clone()]
            || supplied.phrase.situations.iter().any(|s| {
                s.audio_id.is_some() || s.partner.as_ref().is_some_and(|p| p.audio_id.is_some())
            })
        {
            return Err("Supply one new English phrase with matching original reference metadata and text-only situations".into());
        }
        Ok(supplied)
    }

    pub fn submission(
        &self,
        submission_id: Uuid,
        media: &MediaAsset,
    ) -> Result<LessonSubmission, String> {
        if !media.available || media.metadata != self.metadata {
            return Err("Server did not acknowledge the supplied original audio metadata".into());
        }
        let context = serde_json::to_string(self).map_err(|_| "Cannot encode supplied lesson")?;
        if context.len() > 8_000 {
            return Err("Prepared lesson exceeds the inbox context limit".into());
        }
        Ok(LessonSubmission {
            submission_id,
            item_id: self.lesson.source_id.clone(),
            expected_generation: None,
            language: Language::En,
            title: self.lesson.title.clone(),
            context,
            note: String::new(),
            phrases: vec![LessonPhraseInput {
                source_id: self.phrase.source_id.clone(),
                text: self.phrase.text.clone(),
                reference_audio_id: Some(media.id),
                audio_sha256: Some(media.sha256.clone()),
                expected_revision: None,
            }],
        })
    }
}

pub fn output(claim: &JobClaim) -> Result<LessonOutput, String> {
    let supplied = SuppliedLesson::parse(&claim.input.context)?;
    let [input] = claim.input.phrases.as_slice() else {
        return Err("Supplied-lesson worker accepts exactly one phrase".into());
    };
    if input.source_id != supplied.phrase.source_id
        || input.text != supplied.phrase.text
        || claim.input.item_id != supplied.lesson.source_id
        || claim.input.title != supplied.lesson.title
        || claim.profile.revision != claim.receipt.profile_revision
        || claim.guide.trim().is_empty()
    {
        return Err(
            "Supplied preparation does not match the accepted input and pinned context".into(),
        );
    }
    let mut phrase = supplied.phrase;
    phrase.reference_audio_id = input
        .reference_audio_id
        .ok_or("Reference audio is missing")?;
    phrase.expected_revision = input.expected_revision;
    Ok(LessonOutput {
        fence: claim.fence.clone(),
        publication_id: Uuid::new_v5(&claim.receipt.generation_id, b"supplied-lesson-v1"),
        adapter: "supplied-lesson-v1".into(),
        package: PreparedEnglishLesson {
            schema_version: EnglishPackageVersion::EnglishPhrasesV1,
            namespace: claim.receipt.namespace.clone(),
            source_revision: claim.receipt.generation.to_string(),
            language: Language::En,
            profile_revision: claim.receipt.profile_revision,
            guide_version: claim.receipt.guide_version.clone(),
            assessment_version: supplied.assessment_version,
            lesson: supplied.lesson,
            phrases: vec![phrase],
        },
    })
}
