from fastapi import APIRouter, HTTPException

from phrase_gym.server.contracts import CleanupStatus, DeletionResult
from phrase_gym.server.deletion import PhraseDeletion


def deletion_routes(deletion: PhraseDeletion) -> APIRouter:
    router = APIRouter(prefix="/api")

    @router.delete("/phrases/{phrase_id}")
    def delete_phrase(phrase_id: str) -> DeletionResult:
        result = deletion.delete(phrase_id)
        if not result:
            raise HTTPException(404, "Phrase not found; it may already be deleted")
        return result

    @router.get("/cleanup")
    def cleanup_status() -> CleanupStatus:
        return deletion.cleanup.status()

    @router.post("/cleanup/retry")
    def retry_cleanup() -> CleanupStatus:
        return deletion.cleanup.retry()

    return router
