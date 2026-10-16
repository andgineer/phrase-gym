use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::errors::ErrorBody;
use phrase_gym_contracts::exposures::{ExposureInput, ExposureKind, ExposureReceipt};
use utoipa::OpenApi;

use crate::routes;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Phrase Gym Server",
        description = "Rust server API during the migration"
    ),
    paths(routes::exposures::expose),
    components(schemas(ErrorBody, ExposureInput, ExposureKind, ExposureReceipt, Language))
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
