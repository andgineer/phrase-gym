use axum::Json;
use axum::extract::State;
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::library::{Dialogue, DialoguePage, LessonPage, Phrase, PhrasePage};
use uuid::Uuid;

use super::params::{PageQuery, RevisionQuery, SearchQuery};
use crate::error::ApiError;
use crate::extract::{ApiPath, ApiQuery};
use crate::state::AppState;

#[utoipa::path(
    get, path = "/api/v1/lessons", operation_id = "list_lessons", tag = "library",
    params(SearchQuery),
    responses(
        (status = 200, description = "Lessons in the learning language", body = LessonPage),
        (status = 422, description = "Invalid query", body = ErrorBody),
    ),
)]
pub(crate) async fn lessons(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<SearchQuery>,
) -> Result<Json<LessonPage>, ApiError> {
    let paging = query.paging()?;
    let library = state.library.clone();
    let page = state
        .blocking(move || library.lessons(query.language, &query.q, paging))
        .await??;
    Ok(Json(page))
}

#[utoipa::path(
    get, path = "/api/v1/phrases/search", operation_id = "search_phrases", tag = "library",
    params(SearchQuery),
    responses(
        (status = 200, description = "Matching phrases in the learning language", body = PhrasePage),
        (status = 422, description = "Invalid query", body = ErrorBody),
    ),
)]
pub(crate) async fn search(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<SearchQuery>,
) -> Result<Json<PhrasePage>, ApiError> {
    let paging = query.paging()?;
    let library = state.library.clone();
    let page = state
        .blocking(move || library.phrases(query.language, &query.q, None, paging))
        .await??;
    Ok(Json(page))
}

#[utoipa::path(
    get, path = "/api/v1/lessons/{lesson_id}/phrases", operation_id = "lesson_phrases",
    tag = "library",
    params(("lesson_id" = Uuid, Path), PageQuery),
    responses(
        (status = 200, description = "Lesson members in lesson order", body = PhrasePage),
        (status = 404, description = "Lesson not found in this learning language", body = ErrorBody),
        (status = 422, description = "Invalid query", body = ErrorBody),
    ),
)]
pub(crate) async fn lesson_phrases(
    State(state): State<AppState>,
    ApiPath(lesson): ApiPath<Uuid>,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<PhrasePage>, ApiError> {
    let paging = query.paging()?;
    let library = state.library.clone();
    let page = state
        .blocking(move || library.phrases(query.language, "", Some(lesson), paging))
        .await??;
    Ok(Json(page))
}

#[utoipa::path(
    get, path = "/api/v1/phrases/{target_id}", operation_id = "read_phrase", tag = "library",
    params(("target_id" = Uuid, Path), RevisionQuery),
    responses(
        (status = 200, description = "Phrase at its current or requested revision", body = Phrase),
        (status = 404, description = "Phrase or revision not found", body = ErrorBody),
        (status = 422, description = "Invalid query", body = ErrorBody),
    ),
)]
pub(crate) async fn phrase(
    State(state): State<AppState>,
    ApiPath(target): ApiPath<Uuid>,
    ApiQuery(query): ApiQuery<RevisionQuery>,
) -> Result<Json<Phrase>, ApiError> {
    let revision = query.revision()?;
    let library = state.library.clone();
    let phrase = state
        .blocking(move || library.phrase(target, query.language, revision))
        .await??;
    Ok(Json(phrase))
}

#[utoipa::path(
    get, path = "/api/v1/dialogues", operation_id = "list_dialogues", tag = "library",
    params(PageQuery),
    responses(
        (status = 200, description = "Current dialogue revisions", body = DialoguePage),
        (status = 422, description = "Invalid query", body = ErrorBody),
    ),
)]
pub(crate) async fn dialogues(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<DialoguePage>, ApiError> {
    let paging = query.paging()?;
    let library = state.library.clone();
    let page = state
        .blocking(move || library.dialogues(query.language, paging))
        .await??;
    Ok(Json(page))
}

#[utoipa::path(
    get, path = "/api/v1/dialogues/{dialogue_id}", operation_id = "read_dialogue",
    tag = "library",
    params(("dialogue_id" = Uuid, Path), RevisionQuery),
    responses(
        (status = 200, description = "Dialogue at its current or requested revision", body = Dialogue),
        (status = 404, description = "Dialogue not found in this learning language", body = ErrorBody),
        (status = 422, description = "Invalid query", body = ErrorBody),
    ),
)]
pub(crate) async fn dialogue(
    State(state): State<AppState>,
    ApiPath(dialogue): ApiPath<Uuid>,
    ApiQuery(query): ApiQuery<RevisionQuery>,
) -> Result<Json<Dialogue>, ApiError> {
    let revision = query.revision()?;
    let library = state.library.clone();
    let found = state
        .blocking(move || library.dialogue(dialogue, query.language, revision))
        .await??;
    Ok(Json(found))
}
