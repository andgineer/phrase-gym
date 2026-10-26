import os
from pathlib import Path

from fastapi import FastAPI, HTTPException
from fastapi.staticfiles import StaticFiles
from starlette.middleware.trustedhost import TrustedHostMiddleware

from phrase_gym.api_recordings import recording_routes
from phrase_gym.api_study import study_routes
from phrase_gym.boundary import LocalBoundary, trusted_hosts
from phrase_gym.contracts import ImportRequest, ImportStatus
from phrase_gym.importer import ImportConflictError, Importer
from phrase_gym.speech import configured_speech
from phrase_gym.storage import Database
from phrase_gym.study import StudyService


def create_app() -> FastAPI:
    remote_origin = os.environ.get("PHRASE_GYM_TAILSCALE_ORIGIN", "")
    data_dir = Path(os.environ.get("PHRASE_GYM_DATA_DIR", ".data")).resolve()
    database = Database(data_dir)
    importer = Importer(database, configured_speech())
    app = FastAPI(title="Phrase Gym", version="1.0.0")
    app.state.database = database
    app.state.importer = importer
    app.include_router(recording_routes(database))
    app.include_router(study_routes(StudyService(database)))
    app.add_middleware(LocalBoundary, tailscale_origin=remote_origin)
    app.add_middleware(
        TrustedHostMiddleware,
        allowed_hosts=trusted_hosts(remote_origin),
    )

    @app.get("/api/health")
    def health() -> dict[str, str]:
        return {"status": "ok"}

    @app.post("/api/imports", status_code=202)
    async def submit_import(request: ImportRequest) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.submit(request.package))
        except ImportConflictError as error:
            raise HTTPException(409, str(error)) from error

    @app.get("/api/imports/{package_id}")
    def import_status(package_id: str) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.status(package_id))
        except KeyError as error:
            raise HTTPException(404, "Import not found") from error

    @app.post("/api/imports/{package_id}/retry", status_code=202)
    async def retry_import(package_id: str) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.retry(package_id))
        except KeyError as error:
            raise HTTPException(404, "Import not found") from error
        except ImportConflictError as error:
            raise HTTPException(409, str(error)) from error

    static = Path(__file__).parent / "static"
    if static.is_dir():
        app.mount("/", StaticFiles(directory=static, html=True), name="client")
    return app
