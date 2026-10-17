use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::content::{
    Chunk, Cue, DialogueInput, DialogueTurn, Explanation, TargetInput, TargetKind,
};
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::exposures::{ExposureInput, ExposureKind, ExposureReceipt};
use phrase_gym_contracts::library::{
    AudioStatus, Dialogue, DialoguePage, Lesson, LessonPage, Phrase, PhrasePage,
};
use phrase_gym_contracts::media::{
    MediaAsset, MediaMetadata, MediaOrigin, MediaRole, MediaUpload, Reconciliation, ReferenceMime,
    SettingValue,
};
use phrase_gym_contracts::readiness::DueSummary;
use utoipa::OpenApi;

use crate::routes;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Phrase Gym Server",
        description = "Rust server API during the migration"
    ),
    paths(
        routes::exposures::expose,
        routes::library::lessons,
        routes::library::search,
        routes::library::lesson_phrases,
        routes::library::phrase,
        routes::library::dialogues,
        routes::library::dialogue,
        routes::readiness::readiness,
        routes::media::upload,
        routes::media::metadata,
        routes::media::play,
        routes::media::reconcile,
    ),
    components(schemas(
        AudioStatus,
        Chunk,
        Cue,
        Dialogue,
        DialogueInput,
        DialoguePage,
        DialogueTurn,
        DueSummary,
        ErrorBody,
        Explanation,
        ExposureInput,
        ExposureKind,
        ExposureReceipt,
        Language,
        Lesson,
        LessonPage,
        MediaAsset,
        MediaMetadata,
        MediaOrigin,
        MediaRole,
        MediaUpload,
        Phrase,
        PhrasePage,
        Reconciliation,
        ReferenceMime,
        SettingValue,
        TargetInput,
        TargetKind,
    ))
)]
struct ApiDoc;

pub fn document() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn document_json() -> String {
    let mut text = document()
        .to_pretty_json()
        .unwrap_or_else(|_| unreachable!("generated OpenAPI documents serialize"));
    text.push('\n');
    text
}
