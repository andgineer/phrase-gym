from uuid import UUID

from fastapi import APIRouter, Request

from phrase_gym.contracts.handoff import HandoffEnvelope, HandoffReceipt
from phrase_gym.contracts.preparation import PreparationJob, SourcePackage

router = APIRouter(prefix="/api/v1")


@router.post("/preparation", response_model=PreparationJob)
def enqueue(source: SourcePackage, request: Request):
    return request.app.state.preparation.enqueue(source, request.app.state.preparation_identity)


@router.get("/preparation/{job_id}", response_model=PreparationJob)
def job_status(job_id: UUID, request: Request):
    return request.app.state.preparation.get(job_id)


@router.post("/preparation/{job_id}/retry", response_model=PreparationJob)
def retry(job_id: UUID, request: Request):
    return request.app.state.preparation.retry(job_id)


@router.post("/handoffs", response_model=HandoffReceipt)
def receive(envelope: HandoffEnvelope, request: Request):
    return request.app.state.handoff.receive(envelope)


@router.get("/handoffs/{export_id}", response_model=HandoffReceipt)
def handoff_status(export_id: UUID, request: Request):
    return request.app.state.handoff.status(export_id)
