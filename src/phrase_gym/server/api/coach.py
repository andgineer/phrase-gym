from fastapi import APIRouter, HTTPException

from phrase_gym.server.coach_contracts import (
    CoachFailure,
    CoachInbox,
    CoachRequest,
    CoachResult,
    CoachSubmission,
    CoachWork,
)
from phrase_gym.server.coach_inbox import CoachResults
from phrase_gym.server.coach_requests import CoachRequests
from phrase_gym.server.import_contracts import ImportStatus
from phrase_gym.server.reference_audio import ImportConflictError

NOT_FOUND = "Coach request not found"


def coach_inbox_routes(results: CoachResults) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.get("/coach/inbox")
    def inbox() -> CoachInbox:
        return results.inbox()

    @router.post("/coach/requests/{request_id}/dismiss")
    def dismiss(request_id: str) -> None:
        try:
            results.dismiss(request_id)
        except KeyError as error:
            raise HTTPException(404, NOT_FOUND) from error

    @router.post("/coach/requests/{request_id}/retry")
    def retry(request_id: str) -> CoachSubmission:
        try:
            return CoachSubmission.model_validate(results.retry(request_id))
        except KeyError as error:
            raise HTTPException(404, NOT_FOUND) from error
        except ValueError as error:
            raise HTTPException(409, str(error)) from error

    return router


def coach_routes(requests: CoachRequests) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.post("/coach/requests")
    def submit_text(request: CoachRequest) -> CoachSubmission:
        try:
            return CoachSubmission.model_validate(
                requests.submit(request.text, request.learning_language),
            )
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
            return ImportStatus.model_validate(requests.publish(package_id, result))
        except KeyError as error:
            raise HTTPException(404, NOT_FOUND) from error
        except ImportConflictError as error:
            raise HTTPException(409, str(error)) from error

    @router.post("/imports/{package_id}/failure")
    def publish_failure(package_id: str, failure: CoachFailure) -> ImportStatus:
        try:
            return ImportStatus.model_validate(
                requests.fail(package_id, failure.error, failure.retry_helps),
            )
        except KeyError as error:
            raise HTTPException(404, NOT_FOUND) from error

    return router
