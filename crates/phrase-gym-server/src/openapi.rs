use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::content::{
    Chunk, Cue, DialogueInput, DialogueTurn, Explanation, TargetInput, TargetKind,
};
use phrase_gym_contracts::english::{
    EnglishPackageVersion, PartnerPrompt, PhraseFamiliarity, PhraseHelp, PhraseSituation,
    PreparedEnglishLesson, PreparedEnglishPhrase,
};
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::exposures::{ExposureInput, ExposureKind, ExposureReceipt};
use phrase_gym_contracts::inbox::JobMediaRequest;
use phrase_gym_contracts::inbox::{
    ClaimRequest, DeliveryStatus, JobClaim, JobFailure, JobFence, LessonOutput, LessonPhraseInput,
    LessonReceipt, LessonStatus, LessonSubmission, PublicationReceipt,
};
use phrase_gym_contracts::library::{
    AudioStatus, Dialogue, DialoguePage, Lesson, LessonPage, Phrase, PhrasePage,
};
use phrase_gym_contracts::media::{
    MediaAsset, MediaMetadata, MediaOrigin, MediaRole, MediaUpload, Reconciliation, ReferenceMime,
    SettingValue,
};
use phrase_gym_contracts::practice::{
    AttemptEvidence, EventBatch, EventReceipt, Modality, OriginalResponse, PracticeInput,
    PracticeKind, RecallRating,
};
use phrase_gym_contracts::profiles::{
    LanguagePreferences, PersonalProfile, ProfileRestore, ProfileRevision, ProfileUpdate,
};
use phrase_gym_contracts::readiness::DueSummary;
use phrase_gym_contracts::recordings::{RecordingInput, RecordingReceipt};
use phrase_gym_contracts::sessions::{
    Eligibility, OpenStatus, PlannedTarget, PlannerVersion, PracticeBatchReceipt, PracticeSession,
    SessionManifest, SessionStart, SessionStatus, SessionStep, SessionTransition,
};
use phrase_gym_contracts::settings::{Settings, SettingsUpdate, UiLanguageMode};
use utoipa::OpenApi;

use crate::routes;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Phrase Gym Server",
        description = "Rust server API during the migration"
    ),
    paths(
        routes::inbox::receipt,
        routes::inbox::submit,
        routes::inbox::status,
        routes::inbox::cancel,
        routes::inbox::retry,
        routes::jobs::audio,
        routes::jobs::claim,
        routes::jobs::renew,
        routes::jobs::fail,
        routes::jobs::stage,
        routes::jobs::publish,
        routes::jobs::receipt,
        routes::profiles::current,
        routes::profiles::revision,
        routes::profiles::update,
        routes::profiles::restore,
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
        routes::sessions::start,
        routes::sessions::current,
        routes::sessions::read,
        routes::sessions::transition,
        routes::practice::submit,
        routes::practice::receipts,
        routes::recordings::store,
        routes::recordings::list,
        routes::recordings::play,
        routes::settings::read,
        routes::settings::replace,
    ),
    components(schemas(
        JobMediaRequest,
        LessonPhraseInput,
        LessonSubmission,
        LessonReceipt,
        PublicationReceipt,
        DeliveryStatus,
        LessonStatus,
        ClaimRequest,
        JobFence,
        JobClaim,
        LessonOutput,
        JobFailure,
        EnglishPackageVersion,
        PartnerPrompt,
        PhraseSituation,
        PhraseHelp,
        PhraseFamiliarity,
        PreparedEnglishPhrase,
        PreparedEnglishLesson,
        PersonalProfile,
        LanguagePreferences,
        ProfileRevision,
        ProfileUpdate,
        ProfileRestore,
        AttemptEvidence,
        AudioStatus,
        Chunk,
        Cue,
        Dialogue,
        DialogueInput,
        DialoguePage,
        DialogueTurn,
        DueSummary,
        Eligibility,
        ErrorBody,
        EventBatch,
        EventReceipt,
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
        Modality,
        OpenStatus,
        OriginalResponse,
        Phrase,
        PhrasePage,
        PlannedTarget,
        PlannerVersion,
        PracticeBatchReceipt,
        PracticeInput,
        PracticeKind,
        PracticeSession,
        RecallRating,
        Reconciliation,
        RecordingInput,
        RecordingReceipt,
        ReferenceMime,
        SessionManifest,
        SessionStart,
        SessionStatus,
        SessionStep,
        SessionTransition,
        SettingValue,
        Settings,
        SettingsUpdate,
        TargetInput,
        TargetKind,
        UiLanguageMode,
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
