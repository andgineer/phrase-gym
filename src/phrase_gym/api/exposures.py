from fastapi import APIRouter, Request

from phrase_gym.contracts.exposures import ExposureInput, ExposureReceipt

router = APIRouter(prefix="/api/v1")


@router.post("/exposures", response_model=ExposureReceipt)
def exposure(payload: ExposureInput, request: Request):
    return request.app.state.exposures.expose(payload)
