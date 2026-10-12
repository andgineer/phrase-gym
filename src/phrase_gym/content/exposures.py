from collections.abc import Callable
from datetime import datetime

from phrase_gym.clock import utc_now
from phrase_gym.contracts.exposures import ExposureInput, ExposureReceipt
from phrase_gym.errors import DomainError
from phrase_gym.storage import Database, ExposureRow, Target, TargetRevision


class ExposureService:
    def __init__(self, database: Database, clock: Callable[[], datetime] = utc_now):
        self.db = database
        self.clock = clock

    def expose(self, exposure: ExposureInput) -> ExposureReceipt:
        with self.db.write() as session:
            previous = session.get(ExposureRow, str(exposure.id))
            if previous:
                if previous.payload != exposure.model_dump_json():
                    raise DomainError("Exposure ID already used with a different payload")
                return ExposureReceipt(id=exposure.id, received_at=previous.received_at)
            target = session.get(Target, str(exposure.target_id))
            revision = session.get(TargetRevision, (str(exposure.target_id), exposure.revision))
            if target is None or target.language != exposure.language or revision is None:
                raise DomainError("Exposure references an unknown phrase/language/revision", 422)
            received_at = self.clock().isoformat()
            session.add(
                ExposureRow(
                    id=str(exposure.id),
                    target_id=str(exposure.target_id),
                    revision=exposure.revision,
                    payload=exposure.model_dump_json(),
                    received_at=received_at,
                ),
            )
            return ExposureReceipt(id=exposure.id, received_at=received_at)
