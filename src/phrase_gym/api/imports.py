from uuid import UUID

from fastapi import APIRouter, Request

from phrase_gym.contracts.content import ManualPackage
from phrase_gym.contracts.imports import ImportReport, PublishRequest

router = APIRouter(prefix="/api/v1")


@router.post("/imports/validate", response_model=ImportReport)
def validate_import(package: ManualPackage, request: Request):
    return request.app.state.imports.validate(package)


@router.post("/imports", response_model=ImportReport)
def stage_import(package: ManualPackage, request: Request):
    return request.app.state.imports.stage(package)


@router.get("/imports/{import_id}", response_model=ImportReport)
def get_import(import_id: UUID, request: Request):
    return request.app.state.imports.staged(import_id)


@router.post("/imports/{import_id}/publish", response_model=ImportReport)
def publish_import(import_id: UUID, approval: PublishRequest, request: Request):
    assert approval.reviewed
    return request.app.state.imports.publish(import_id)
