"""Disposable P2 browser fixture server; no user data or external providers."""

import argparse
import base64
import io
import math
import struct
import tempfile
import wave
from pathlib import Path

import uvicorn

from phrase_gym.api.app import create_app
from phrase_gym.contracts.content import ManualPackage
from phrase_gym.contracts.media import MediaMetadata, MediaUpload
from phrase_gym.config import Config
from phrase_gym.content.imports import ImportService
from phrase_gym.content.validation import PackageValidator
from phrase_gym.media.service import MediaService
from phrase_gym.storage import Database

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


def run() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8941)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="phrase-gym-browser-") as directory:
        config = Config(Path(directory), client_dir=ROOT / "frontend/dist")
        database = Database(config.data_dir)
        database.migrate()
        media = MediaService(database, config.data_dir)
        content = ImportService(database, PackageValidator(media.available))
        for name in ("manual-en.json", "manual-sr.json"):
            package = ManualPackage.model_validate_json(
                (ROOT / "fixtures" / name).read_text(encoding="utf-8")
            )
            target = package.targets[0]
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
        database.close()
        uvicorn.run(create_app(config), host="127.0.0.1", port=args.port, log_level="warning")


if __name__ == "__main__":
    run()
