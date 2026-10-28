from fastapi import APIRouter, HTTPException

from phrase_gym.server.contracts import CoachFailure, CoachResult, CoachWork, ImportStatus
from phrase_gym.server.importer import Importer
from phrase_gym.server.reference_audio import ImportConflictError


def coach_routes(importer: Importer) -> APIRouter:
    router = APIRouter(prefix="/api/imports")

    @router.post("/coach/next")
    def next_work() -> CoachWork | None:
        work = importer.next_work()
        return CoachWork.model_validate(work) if work else None

    @router.post("/{package_id}/result")
    def publish_result(package_id: str, result: CoachResult) -> ImportStatus:
        try:
            return ImportStatus.model_validate(
                importer.publish(package_id, result.result_id, result.references),
            )
        except KeyError as error:
            raise HTTPException(404, "Import not found") from error
        except ImportConflictError as error:
            raise HTTPException(409, str(error)) from error

    @router.post("/{package_id}/failure")
    def publish_failure(package_id: str, failure: CoachFailure) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.fail(package_id, failure.error))
        except KeyError as error:
            raise HTTPException(404, "Import not found") from error

    return router
