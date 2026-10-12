import base64
from uuid import uuid4

import pytest

from tests.integration.helpers import ROOT, upload


@pytest.mark.parametrize(
    "suffix,mime", [("mp3", "audio/mpeg"), ("ogg", "audio/ogg"), ("m4a", "audio/mp4")]
)
def test_compressed_container_retains_original_bytes_and_rejects_mismatched_mime(
    client, suffix, mime
):
    audio = (ROOT / f"tests/fixtures/media/tone.{suffix}").read_bytes()
    metadata = upload(client).json()["metadata"]
    payload = {
        "metadata": metadata,
        "audio_base64": base64.b64encode(audio).decode(),
        "mime_type": mime,
    }
    response = client.post("/api/v1/media", json=payload)
    assert response.status_code == 200, response.text
    asset = response.json()
    assert asset["available"] and asset["byte_length"] == len(audio)
    assert client.post("/api/v1/media", json=payload).json()["id"] == asset["id"]
    delivery = client.get(f"/api/v1/media/{asset['id']}")
    assert delivery.content == audio
    assert delivery.headers["content-type"] == mime
    for other in {"audio/wav", "audio/mpeg", "audio/ogg", "audio/mp4"} - {mime}:
        assert client.post("/api/v1/media", json={**payload, "mime_type": other}).status_code == 422


@pytest.mark.parametrize(
    "metadata",
    [
        {"locale": "sr-RS"},
        {"origin": "actor", "performer": None},
        {"origin": "course", "performer": ""},
        {"origin": "generated", "engine": "test", "voice": "test", "speech_text": None},
        {"origin": "generated", "engine": None, "voice": "test", "speech_text": "Test"},
        {"origin": "generated", "engine": "test", "voice": None, "speech_text": "Test"},
    ],
)
def test_incomplete_or_wrong_language_provenance_is_rejected(client, metadata):
    assert upload(client, **metadata).status_code == 422


def test_malformed_base64_and_unknown_media_are_rejected(client):
    metadata = upload(client).json()["metadata"]
    response = client.post(
        "/api/v1/media",
        json={"metadata": metadata, "mime_type": "audio/wav", "audio_base64": "%%%invalid%%%"},
    )
    assert response.status_code == 422
    assert "valid base64" in response.json()["detail"]
    assert client.get(f"/api/v1/media/{uuid4()}").status_code == 404
