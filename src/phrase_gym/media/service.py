"""Original reference bytes, verified hash identities and crash reconciliation."""

import base64
import binascii
import hashlib
import io
import os
import tempfile
import time
import wave
from pathlib import Path
from uuid import UUID, uuid5

from mutagen import MutagenError
from mutagen.mp3 import MP3
from mutagen.mp4 import MP4
from mutagen.oggvorbis import OggVorbis
from sqlalchemy import select

from phrase_gym.contracts.media import MediaAsset, MediaMetadata, MediaUpload, Reconciliation
from phrase_gym.errors import DomainError
from phrase_gym.serialization import canonical_json
from phrase_gym.storage import Database
from phrase_gym.tables import MediaRow

MEDIA_NAMESPACE = UUID("1a6d9d67-c9b9-4d73-9e38-6dd05a3c1d66")
MAX_AUDIO_BYTES = 12 * 1024 * 1024
STAGING_RETENTION_SECONDS = 3600


def validate_audio(audio: bytes, mime_type: str) -> None:
    if not audio or len(audio) > MAX_AUDIO_BYTES:
        raise DomainError("Reference audio must be nonempty and at most 12 MiB", 422)
    try:
        if mime_type == "audio/wav":
            with wave.open(io.BytesIO(audio)) as wav:
                expected = wav.getnframes() * wav.getnchannels() * wav.getsampwidth()
                if expected <= 0 or len(wav.readframes(wav.getnframes())) != expected:
                    raise DomainError("WAV has empty or truncated audio", 422)
        else:
            parsers = {"audio/mpeg": MP3, "audio/mp4": MP4, "audio/ogg": OggVorbis}
            parsed = parsers[mime_type](io.BytesIO(audio))
            if parsed.info is None or parsed.info.length <= 0:
                raise DomainError("Audio has no playable duration", 422)
    except (MutagenError, wave.Error, EOFError, KeyError, ValueError) as error:
        raise DomainError(
            "Invalid audio or MIME type does not match its container",
            422,
        ) from error


class MediaService:
    def __init__(self, database: Database, data_dir: Path):
        self.db = database
        self.root = data_dir / "media"
        self.staging = data_dir / "staging"
        self.root.mkdir(exist_ok=True)
        self.staging.mkdir(exist_ok=True)

    def path(self, row: MediaRow) -> Path:
        return self.root / row.sha256

    def available(self, row: MediaRow) -> bool:
        path = self.path(row)
        if not path.is_file() or path.stat().st_size != row.byte_length:
            return False
        return hashlib.sha256(path.read_bytes()).hexdigest() == row.sha256

    def model(self, row: MediaRow) -> MediaAsset:
        return MediaAsset(
            id=UUID(row.id),
            sha256=row.sha256,
            byte_length=row.byte_length,
            mime_type=row.mime_type,
            metadata=MediaMetadata.model_validate_json(row.metadata_json),
            available=self.available(row),
        )

    def upload(self, upload: MediaUpload) -> MediaAsset:
        try:
            audio = base64.b64decode(upload.audio_base64, validate=True)
        except (ValueError, binascii.Error) as error:
            raise DomainError("Audio must be valid base64", 422) from error
        validate_audio(audio, upload.mime_type)
        sha256 = hashlib.sha256(audio).hexdigest()
        metadata_json = canonical_json(upload.metadata.model_dump_json())
        identity = "|".join((sha256, upload.mime_type, metadata_json))
        media_id = str(uuid5(MEDIA_NAMESPACE, identity))
        with tempfile.NamedTemporaryFile(dir=self.staging, suffix=".part", delete=False) as stream:
            temporary = Path(stream.name)
            stream.write(audio)
            stream.flush()
            os.fsync(stream.fileno())
        try:
            with self.db.write() as session:
                os.replace(temporary, self.root / sha256)
                # Preserve IDs already bound by packages written before canonical serialization.
                row = next(
                    (
                        candidate
                        for candidate in session.scalars(
                            select(MediaRow).where(
                                MediaRow.sha256 == sha256,
                                MediaRow.mime_type == upload.mime_type,
                            ),
                        )
                        if canonical_json(candidate.metadata_json) == metadata_json
                    ),
                    None,
                )
                if row is None:
                    row = MediaRow(
                        id=media_id,
                        sha256=sha256,
                        byte_length=len(audio),
                        mime_type=upload.mime_type,
                        metadata_json=metadata_json,
                    )
                    session.add(row)
                session.flush()
                result = self.model(row)
        finally:
            temporary.unlink(missing_ok=True)
        return result

    def get(self, media_id: UUID) -> MediaAsset:
        with self.db.read() as session:
            row = session.get(MediaRow, str(media_id))
            if row is None:
                raise DomainError("Reference media not found", 404)
            return self.model(row)

    def delivery(self, media_id: UUID) -> tuple[Path, MediaAsset]:
        with self.db.read() as session:
            row = session.get(MediaRow, str(media_id))
            if row is None:
                raise DomainError("Reference media not found", 404)
            asset = self.model(row)
            if not asset.available:
                raise DomainError(
                    "Reference audio is missing or damaged; re-upload the original clip",
                    503,
                )
            return self.path(row), asset

    def reconcile(self) -> Reconciliation:
        removed = 0
        for path in self.staging.glob("*.part"):
            if path.stat().st_mtime < time.time() - STAGING_RETENTION_SECONDS:
                path.unlink(missing_ok=True)
                removed += 1
        with self.db.read() as session:
            rows = session.scalars(select(MediaRow)).all()
            referenced = {row.sha256 for row in rows}
            missing = [UUID(row.id) for row in rows if not self.available(row)]
            orphans = sorted(
                path.name
                for path in self.root.iterdir()
                if path.is_file() and path.name not in referenced
            )
        return Reconciliation(
            missing_ids=missing,
            orphan_hashes=orphans,
            removed_staging_files=removed,
        )
