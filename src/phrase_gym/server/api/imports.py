from fastapi import APIRouter, HTTPException

from phrase_gym.server.import_contracts import (
    ImportRequest,
    ImportStatus,
)
from phrase_gym.server.import_notices import ImportNotices
from phrase_gym.server.importer import Importer
from phrase_gym.server.reference_audio import ImportConflictError


def import_routes(importer: Importer, notices: ImportNotices) -> APIRouter:
    router = APIRouter(prefix="/api/imports")

    @router.post("", status_code=202)
    async def submit_import(request: ImportRequest) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.submit(request.package, request.references))
        except ImportConflictError as error:
            raise HTTPException(409, str(error)) from error

    @router.get("/{package_id}")
    def import_status(package_id: str) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.status(package_id))
        except KeyError as error:
            raise HTTPException(404, "Import not found") from error

    @router.delete("/{package_id}/notice")
    def dismiss_notice(package_id: str) -> None:
        notices.dismiss(package_id)

    return router
