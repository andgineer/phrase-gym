"""Disposable P2 browser fixture server; no user data or external providers."""

import argparse
import asyncio
import base64
import io
import math
import struct
import tempfile
import wave
from pathlib import Path
from uuid import UUID

import uvicorn
from fastapi import Request
from fastapi.responses import JSONResponse
from sqlalchemy import delete

from phrase_gym.api.app import create_app
from phrase_gym.contracts.content import ManualPackage
from phrase_gym.contracts.media import MediaMetadata, MediaUpload
from phrase_gym.config import Config
from phrase_gym.content.imports import ImportService
from phrase_gym.content.validation import PackageValidator
from phrase_gym.media.service import MediaService
from phrase_gym.storage import Database
from phrase_gym.tables import Base

ROOT = Path(__file__).resolve().parents[1]


def tone() -> bytes:
    buffer = io.BytesIO()
    with wave.open(buffer, "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(16000)
        audio.writeframes(
            b"".join(
                struct.pack("<h", int(6000 * math.sin(2 * math.pi * 440 * i / 16000)))
                for i in range(16000)
            )
        )
    return buffer.getvalue()


def seed_library(database: Database, media: MediaService, revision: str = "1") -> None:
    content = ImportService(database, PackageValidator(media.available))
    for name in ("manual-en.json", "manual-sr.json"):
        package = ManualPackage.model_validate_json(
            (ROOT / "fixtures" / name).read_text(encoding="utf-8")
        )
        package.source_revision = revision
        for target in package.targets:
            if revision != "1":
                target.purpose += " Updated publication."
            metadata = MediaMetadata(
                language=package.language,
                text=target.text,
                role="reference",
                origin="test",
                source="P2 synthetic browser fixture",
                locale=f"{package.language}-{'RS' if package.language == 'sr' else 'US'}",
                preparation_version="1",
            )
            asset = media.upload(
                MediaUpload(
                    metadata=metadata,
                    mime_type="audio/wav",
                    audio_base64=base64.b64encode(tone()).decode(),
                )
            )
            target.reference_audio_id = asset.id
        report = content.stage(package)
        assert report.id is not None
        content.publish(report.id)


def run() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8941)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="phrase-gym-browser-") as directory:
        config = Config(Path(directory), client_dir=ROOT / "frontend/dist")
        app = create_app(config)
        app.state.lose_rating_ack = False
        app.state.recording_ack_gate = None

        @app.middleware("http")
        async def lost_ack(request: Request, call_next):
            recording_gate = (
                app.state.recording_ack_gate
                if request.method == "POST" and request.url.path == "/api/v1/recordings"
                else None
            )
            losing = False
            if request.method == "POST" and request.url.path.endswith("/events"):
                payload = await request.json()
                losing = app.state.lose_rating_ack and any(
                    event["kind"] == "rate" for event in payload["events"]
                )
            response = await call_next(request)
            if recording_gate and response.status_code == 200:
                await recording_gate.wait()
            if losing and response.status_code == 200:
                return JSONResponse({"detail": "Rating acknowledgement lost"}, 503)
            return response

        @app.post("/_test/hold-recording-ack", include_in_schema=False)
        async def hold_recording_ack():
            app.state.recording_ack_gate = asyncio.Event()
            return {"held": True}

        @app.post("/_test/release-recording-ack", include_in_schema=False)
        async def release_recording_ack():
            if app.state.recording_ack_gate:
                app.state.recording_ack_gate.set()
            app.state.recording_ack_gate = None
            return {"held": False}

        @app.post("/_test/lose-rating", include_in_schema=False)
        def lose_rating():
            app.state.lose_rating_ack = True
            return {"armed": True}

        @app.post("/_test/acknowledge-rating", include_in_schema=False)
        def acknowledge_rating():
            app.state.lose_rating_ack = False
            return {"armed": False}

        @app.post("/_test/revise-content", include_in_schema=False)
        def revise_content():
            seed_library(app.state.settings.db, app.state.media, "2")
            return {"revised": True}

        @app.post("/_test/reset", include_in_schema=False)
        async def reset_fixture():
            app.state.lose_rating_ack = False
            if app.state.recording_ack_gate:
                app.state.recording_ack_gate.set()
            app.state.recording_ack_gate = None
            database = app.state.settings.db
            with database.write() as session:
                for table in reversed(Base.metadata.sorted_tables):
                    session.execute(delete(table))
            seed_library(database, app.state.media)
            return {"reset": True}

        @app.post("/_test/missing-media/{media_id}", include_in_schema=False)
        def missing_media(media_id: UUID):
            path, _asset = app.state.media.delivery(media_id)
            path.unlink()
            return {"missing": True}

        uvicorn.run(app, host="127.0.0.1", port=args.port, log_level="warning")


if __name__ == "__main__":
    run()
