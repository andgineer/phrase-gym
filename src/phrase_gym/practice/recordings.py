import base64
import binascii
import hashlib
import json
from collections.abc import Callable
from datetime import datetime
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.orm import Session

from phrase_gym.contracts.practice import (
    AttemptEvidence,
    EventReceipt,
    PracticeInput,
    RecordingInput,
    RecordingReceipt,
)
from phrase_gym.errors import DomainError
from phrase_gym.practice.audio import validate_take
from phrase_gym.practice.tables import AttemptRow, PracticeEvent, RecordingRow
from phrase_gym.serialization import canonical_json
from phrase_gym.storage import Database
from phrase_gym.tables import ExposureRow, TargetRevision

MAX_TAKE_BYTES = 12 * 1024 * 1024
ALLOWED_CONTAINERS = {"audio/webm", "audio/ogg", "audio/mp4", "audio/wav", "audio/mpeg"}


class RecordingService:
    def __init__(self, database: Database, clock: Callable[[], datetime]):
        self.db, self.clock = database, clock

    @staticmethod
    def receipt(row: RecordingRow) -> RecordingReceipt:
        return RecordingReceipt(
            id=UUID(row.id),
            sha256=row.sha256,
            byte_length=len(row.audio),
            captured_at=datetime.fromisoformat(json.loads(row.metadata_json)["captured_at"]),
            received_at=datetime.fromisoformat(row.received_at),
            complete=json.loads(row.metadata_json)["complete"],
        )

    def save(self, payload: RecordingInput) -> RecordingReceipt:
        try:
            audio = base64.b64decode(payload.audio_base64, validate=True)
        except (ValueError, binascii.Error) as error:
            raise DomainError("Take audio must be valid base64", 422) from error
        if not audio or len(audio) > MAX_TAKE_BYTES:
            raise DomainError("A take must contain audio and be at most 12 MiB", 422)
        if payload.mime_type.split(";")[0].lower() not in ALLOWED_CONTAINERS:
            raise DomainError("Unsupported recording container", 422)
        if payload.complete:
            validate_take(audio, payload.mime_type)
        # Interrupted browser containers may lack final decoding metadata; retain original bytes.
        metadata = canonical_json(payload.model_dump_json(exclude={"audio_base64"}))
        sha256 = hashlib.sha256(audio).hexdigest()
        with self.db.write() as transaction:
            previous = transaction.get(RecordingRow, str(payload.id))
            if previous:
                if previous.metadata_json != metadata or previous.sha256 != sha256:
                    raise DomainError("Take ID already has different audio or metadata", 409)
                return self.receipt(previous)
            self.validate_context(transaction, payload)
            row = RecordingRow(
                id=str(payload.id),
                target_id=str(payload.target_id),
                attempt_id=str(payload.attempt_id) if payload.attempt_id else None,
                exposure_id=str(payload.exposure_id) if payload.exposure_id else None,
                metadata_json=metadata,
                audio=audio,
                sha256=sha256,
                received_at=self.clock().isoformat(),
            )
            transaction.add(row)
            return self.receipt(row)

    @staticmethod
    def validate_context(transaction: Session, payload: RecordingInput) -> None:
        if transaction.get(TargetRevision, (str(payload.target_id), payload.revision)) is None:
            raise DomainError("Take content revision not found", 422)
        if payload.attempt_id:
            attempt = transaction.get(AttemptRow, str(payload.attempt_id))
            if attempt is None:
                raise DomainError("Persist the attempt before its take", 409)
            evidence = AttemptEvidence.model_validate_json(attempt.payload)
            if evidence.target_id != payload.target_id or evidence.revision != payload.revision:
                raise DomainError("Take does not match its attempt", 422)
        if payload.capture_event_id:
            event = transaction.get(PracticeEvent, str(payload.capture_event_id))
            if event is None or PracticeInput.model_validate_json(event.payload).kind != "respond":
                raise DomainError("Take requires an acknowledged capture event", 409)
            if EventReceipt.model_validate_json(event.receipt).attempt_id != payload.attempt_id:
                raise DomainError("Capture event does not match its take", 422)
        if payload.exposure_id:
            exposure = transaction.get(ExposureRow, str(payload.exposure_id))
            if (
                not exposure
                or exposure.target_id != str(payload.target_id)
                or exposure.revision != payload.revision
            ):
                raise DomainError("Take does not match its lookup exposure", 422)

    def list_for_target(self, target_id: UUID) -> list[RecordingReceipt]:
        with self.db.read() as transaction:
            rows = [
                self.receipt(row)
                for row in transaction.scalars(
                    select(RecordingRow)
                    .where(
                        RecordingRow.target_id == str(target_id),
                    )
                    .order_by(RecordingRow.received_at, RecordingRow.id),
                )
            ]

            return sorted(rows, key=lambda item: (item.captured_at, item.received_at, str(item.id)))

    def delivery(self, recording_id: UUID) -> tuple[bytes, str, str]:
        with self.db.read() as transaction:
            row = transaction.get(RecordingRow, str(recording_id))
            if row is None:
                raise DomainError("Take not found", 404)
            metadata = json.loads(row.metadata_json)
            return row.audio, metadata["mime_type"], row.sha256
