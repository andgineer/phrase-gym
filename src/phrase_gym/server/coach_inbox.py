import json

from phrase_gym.server.coach_contracts import CoachCorrection, CoachInbox, CoachOutcome
from phrase_gym.server.coach_requests import coach_row, request_language, requeue
from phrase_gym.server.import_notices import clear_notice
from phrase_gym.server.records import ImportRecord, fetch_all
from phrase_gym.server.storage import Database


def _outcome(row: ImportRecord) -> CoachOutcome:
    correction = (
        CoachCorrection.model_validate_json(row.correction_json) if row.correction_json else None
    )
    kind = "corrected" if row.status == "added" else "failed" if row.retry_helps else "rejected"
    return CoachOutcome(
        request_id=row.package_id,
        text=json.loads(row.payload)["text"],
        learning_language=request_language(row),
        outcome=kind,
        reason=row.error,
        correction=correction,
    )


class CoachResults:
    """Finished Coach requests the learner still needs to see: changes, rejections, failures."""

    def __init__(self, database: Database):
        self.database = database

    def inbox(self) -> CoachInbox:
        with self.database.read() as connection:
            rows = fetch_all(
                connection,
                ImportRecord,
                "SELECT * FROM imports WHERE kind='coach' AND dismissed=0 AND "
                "(status='error' OR (status='added' AND correction_json IS NOT NULL)) "
                "ORDER BY created_at DESC, package_id",
            )
        return CoachInbox(items=[_outcome(row) for row in rows])

    def dismiss(self, package_id: str) -> None:
        with self.database.write() as connection:
            coach_row(connection, package_id)
            connection.execute("UPDATE imports SET dismissed=1 WHERE package_id=?", (package_id,))
            clear_notice(connection, package_id)

    def retry(self, package_id: str) -> dict[str, str]:
        with self.database.write() as connection:
            row = coach_row(connection, package_id)
            if row.status != "error" or not row.retry_helps:
                raise ValueError("Only a request that failed for a service reason can be retried.")
            requeue(connection, package_id)
        return {"request_id": package_id, "outcome": "saved"}
