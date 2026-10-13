from collections.abc import Callable
from contextlib import asynccontextmanager
from datetime import datetime
from pathlib import Path

from fastapi import FastAPI, Request
from fastapi.responses import FileResponse, JSONResponse
from starlette.middleware.trustedhost import TrustedHostMiddleware
from starlette.staticfiles import StaticFiles

from phrase_gym.api import exposures, imports, library, media, practice, sessions, settings
from phrase_gym.api.boundary import LocalBoundary
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

    if runtime.client_dir and (runtime.client_dir / "index.html").is_file():
        assets = runtime.client_dir / "assets"
        if assets.is_dir():
            app.mount("/assets", StaticFiles(directory=assets), name="assets")

        @app.get("/sw.js", include_in_schema=False)
        @app.get("/manifest.webmanifest", include_in_schema=False)
        @app.get("/icon.svg", include_in_schema=False)
        @app.get("/icon-192.png", include_in_schema=False)
        @app.get("/icon-512.png", include_in_schema=False)
        @app.get("/apple-touch-icon.png", include_in_schema=False)
        def offline_asset(request: Request):
            return FileResponse(
                Path(runtime.client_dir or ".") / request.url.path.lstrip("/"),
                headers={"Cache-Control": "no-cache"},
            )

        @app.get("/", include_in_schema=False)
        @app.get("/index.html", include_in_schema=False)
        @app.get("/learn", include_in_schema=False)
        @app.get("/learn/{client_path:path}", include_in_schema=False)
        def client():
            return FileResponse(
                Path(runtime.client_dir or ".") / "index.html",
                headers={"Cache-Control": "no-cache"},
            )
    else:

        @app.get("/", include_in_schema=False)
        def no_client():
            return JSONResponse(
                {"detail": "Build the client and pass --client-dir frontend/dist"},
                503,
            )

    return app
