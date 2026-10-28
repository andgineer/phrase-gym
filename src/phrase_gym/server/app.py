import os
from pathlib import Path

from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from starlette.middleware.trustedhost import TrustedHostMiddleware

from phrase_gym.server.api.coach import coach_routes
from phrase_gym.server.api.imports import import_routes
from phrase_gym.server.api.library import library_routes
from phrase_gym.server.api.recordings import recording_routes
from phrase_gym.server.api.review import review_routes
from phrase_gym.server.api.settings import settings_routes
from phrase_gym.server.api.study import study_routes
from phrase_gym.server.boundary import LocalBoundary, trusted_hosts
from phrase_gym.server.importer import Importer
from phrase_gym.server.library import LibraryService
from phrase_gym.server.storage import Database
from phrase_gym.server.study import StudyService


def create_app() -> FastAPI:
    remote_origin = os.environ.get("PHRASE_GYM_TAILSCALE_ORIGIN", "")
    data_dir = Path(os.environ.get("PHRASE_GYM_DATA_DIR", ".data")).resolve()
    database = Database(data_dir)
    importer = Importer(database)
    app = FastAPI(title="Phrase Gym", version="1.0.0")
    app.state.database = database
    app.state.importer = importer
    app.include_router(import_routes(importer))
    app.include_router(coach_routes(importer))
    app.include_router(recording_routes(database))
    study = StudyService(database)
    app.state.study = study
    app.include_router(study_routes(study))
    app.include_router(review_routes(study))
    library = LibraryService(database)
    app.include_router(library_routes(library))
    app.include_router(settings_routes(library))
    app.add_middleware(LocalBoundary, tailscale_origin=remote_origin)
    app.add_middleware(
        TrustedHostMiddleware,
        allowed_hosts=trusted_hosts(remote_origin),
    )

    @app.get("/api/health")
    def health() -> dict[str, str]:
        return {"status": "ok"}

    static = Path(__file__).parent / "static"
    if static.is_dir():
        app.mount("/", StaticFiles(directory=static, html=True), name="client")
    return app
