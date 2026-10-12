from uuid import UUID

from fastapi import APIRouter, Request
from fastapi.responses import FileResponse

from phrase_gym.contracts.media import MediaAsset, MediaUpload, Reconciliation

router = APIRouter(prefix="/api/v1")


@router.post("/media", response_model=MediaAsset)
def upload_media(upload: MediaUpload, request: Request):
    return request.app.state.media.upload(upload)


@router.get("/media/{media_id}/metadata", response_model=MediaAsset)
def media_metadata(media_id: UUID, request: Request):
    return request.app.state.media.get(media_id)


@router.get("/media/{media_id}")
def audio(media_id: UUID, request: Request):
    path, asset = request.app.state.media.delivery(media_id)
    return FileResponse(
        path,
        media_type=asset.mime_type,
        headers={
            "ETag": f'"{asset.sha256}"',
            "Cache-Control": "private, max-age=31536000, immutable",
        },
    )


@router.post("/media/reconcile", response_model=Reconciliation)
def reconcile(request: Request):
    return request.app.state.media.reconcile()
