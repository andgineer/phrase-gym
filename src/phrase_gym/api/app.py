from collections.abc import Callable
from contextlib import asynccontextmanager
from datetime import datetime

from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from starlette.middleware.trustedhost import TrustedHostMiddleware

from phrase_gym.api import exposures, imports, library, media, practice, sessions, settings
from phrase_gym.api.boundary import LocalBoundary
from phrase_gym.api.client import mount_client
from phrase_gym.clock import utc_now
from phrase_gym.config import Config
from phrase_gym.content.catalog import LibraryService
from phrase_gym.content.exposures import ExposureService
from phrase_gym.content.imports import ImportService
from phrase_gym.content.validation import PackageValidator
from phrase_gym.errors import DomainError
from phrase_gym.media.service import MediaService
from phrase_gym.practice.events import PracticeService
from phrase_gym.practice.policy import ReviewPolicy
from phrase_gym.practice.recordings import RecordingService
from phrase_gym.sessions.planning import SessionPlanner
from phrase_gym.sessions.readiness import ReadinessService
from phrase_gym.sessions.service import SessionService
from phrase_gym.settings import SettingsService
from phrase_gym.storage import Database


def create_app(config: Config | None = None, clock: Callable[[], datetime] = utc_now) -> FastAPI:
    runtime = config or Config.from_env()

    @asynccontextmanager
    async def lifespan(application: FastAPI):
        database = Database(runtime.data_dir)
        database.migrate()
        reference_media = MediaService(database, runtime.data_dir)
        application.state.settings = SettingsService(database)
        application.state.imports = ImportService(
            database,
            PackageValidator(reference_media.available),
            clock,
        )
        application.state.library = LibraryService(database, reference_media.available)
        application.state.exposures = ExposureService(database, clock)
        application.state.sessions = SessionService(
            database,
            SessionPlanner(application.state.library),
            clock,
        )
        application.state.readiness = ReadinessService(
            database,
            application.state.sessions.planner,
            clock,
        )
        application.state.practice = PracticeService(
            database,
            application.state.sessions,
            ReviewPolicy(),
            clock,
        )
        application.state.recordings = RecordingService(database, clock)
        application.state.media = reference_media
        try:
            yield
        finally:
            database.close()

    app = FastAPI(title="Phrase Gym", version="1.0.0", lifespan=lifespan)
    app.add_middleware(LocalBoundary, allowed_origins=runtime.allowed_origins)
    app.add_middleware(TrustedHostMiddleware, allowed_hosts=list(runtime.allowed_hosts))

    for routes in (settings, imports, library, exposures, media, sessions, practice):
        app.include_router(routes.router)

    @app.exception_handler(DomainError)
    async def domain_error(_request: Request, error: DomainError):
        return JSONResponse({"detail": str(error)}, status_code=error.status)

    mount_client(app, runtime.client_dir)

    return app
