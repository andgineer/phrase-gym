from fastapi import APIRouter, HTTPException

from phrase_gym.server.contracts import ImportRequest, ImportStatus
from phrase_gym.server.importer import ImportConflictError, Importer


def import_routes(importer: Importer) -> APIRouter:
    router = APIRouter(prefix="/api/imports")

    @router.post("", status_code=202)
    async def submit_import(request: ImportRequest) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.submit(request.package))
        except ImportConflictError as error:
            raise HTTPException(409, str(error)) from error

    @router.get("/{package_id}")
    def import_status(package_id: str) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.status(package_id))
        except KeyError as error:
            raise HTTPException(404, "Import not found") from error

    @router.post("/{package_id}/retry", status_code=202)
    async def retry_import(package_id: str) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.retry(package_id))
        except KeyError as error:
            raise HTTPException(404, "Import not found") from error
        except ImportConflictError as error:
            raise HTTPException(409, str(error)) from error

    return router
