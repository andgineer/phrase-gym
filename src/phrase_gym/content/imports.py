from collections.abc import Callable
from datetime import datetime
from uuid import UUID, uuid4

from sqlalchemy.orm import Session

from phrase_gym.clock import utc_now
from phrase_gym.content import publication
from phrase_gym.content.validation import PackageValidator
from phrase_gym.contracts.content import ManualPackage
from phrase_gym.contracts.imports import ImportReport
from phrase_gym.errors import DomainError
from phrase_gym.storage import Database
from phrase_gym.tables import ImportRow


class ImportService:
    def __init__(
        self,
        database: Database,
        validator: PackageValidator,
        clock: Callable[[], datetime] = utc_now,
    ):
        self.db = database
        self.validator = validator
        self.clock = clock

    def validate(self, package: ManualPackage) -> ImportReport:
        with self.db.read() as session:
            return self.validator.report(session, package)

    def stage(self, package: ManualPackage) -> ImportReport:
        with self.db.write() as session:
            report = self.validator.report(session, package)
            if report.errors:
                return report
            report.id = uuid4()
            report.status = "staged"
            session.add(
                ImportRow(
                    id=str(report.id),
                    payload=package.model_dump_json(),
                    report=report.model_dump_json(),
                    status="staged",
                    created_at=self.clock().isoformat(),
                ),
            )
        return report

    def staged(self, import_id: UUID) -> ImportReport:
        with self.db.read() as session:
            row = session.get(ImportRow, str(import_id))
            if row is None:
                raise DomainError("Import not found", 404)
            return ImportReport.model_validate_json(row.report)

    def publish(self, import_id: UUID) -> ImportReport:
        with self.db.write() as session:
            return self.publish_in(session, import_id)

    def publish_in(self, session: Session, import_id: UUID) -> ImportReport:
        row = session.get(ImportRow, str(import_id))
        if row is None:
            raise DomainError("Import not found", 404)
        staged = ImportReport.model_validate_json(row.report)
        if row.status == "published":
            return staged
        package = ManualPackage.model_validate_json(row.payload)
        current = self.validator.report(session, package)
        if current.errors:
            raise DomainError("; ".join(current.errors))
        if (
            current.changes != staged.changes
            or current.base_content_version != staged.base_content_version
        ):
            raise DomainError(
                "Content changed after staging; validate and stage the package again",
            )
        target_ids = publication.publish_targets(session, package, staged.changes, self.clock)
        session.flush()
        publication.publish_lessons(session, package, target_ids)
        publication.publish_dialogues(session, package, target_ids)
        staged.status = "published"
        row.status = "published"
        row.report = staged.model_dump_json()
        return staged
