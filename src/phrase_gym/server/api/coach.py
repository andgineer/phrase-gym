from fastapi import APIRouter, HTTPException

from phrase_gym.server.coach_contracts import (
    CoachFailure,
    CoachRequest,
    CoachResult,
    CoachSubmission,
    CoachWork,
)
from phrase_gym.server.coach_requests import CoachRequests
from phrase_gym.server.import_contracts import ImportStatus
from phrase_gym.server.reference_audio import ImportConflictError


def coach_routes(requests: CoachRequests) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.post("/coach/requests")
    def submit_text(request: CoachRequest) -> CoachSubmission:
        try:
            return CoachSubmission.model_validate(requests.submit(request.text))
        except ValueError as error:
            status = 409 if isinstance(error, ImportConflictError) else 422
            raise HTTPException(status, str(error)) from error

    @router.post("/imports/coach/next")
    def next_work() -> CoachWork | None:
        work = requests.next_work()
        return CoachWork.model_validate(work) if work else None

    @router.post("/imports/{package_id}/result")
    def publish_result(package_id: str, result: CoachResult) -> ImportStatus:
        try:
            return ImportStatus.model_validate(
                requests.publish(package_id, result.result_id, result.phrase, result.reference),
            )
        except KeyError as error:
            raise HTTPException(404, "Coach request not found") from error
        except ImportConflictError as error:
            raise HTTPException(409, str(error)) from error

    @router.post("/imports/{package_id}/failure")
    def publish_failure(package_id: str, failure: CoachFailure) -> ImportStatus:
        try:
            return ImportStatus.model_validate(
                requests.fail(package_id, failure.error, failure.retry_helps),
            )
        except KeyError as error:
            raise HTTPException(404, "Coach request not found") from error

    return router
