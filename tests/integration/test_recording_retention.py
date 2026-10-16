import base64
import hashlib
from uuid import uuid4

import pytest

from tests.integration.helpers import ROOT
from tests.integration.practice_helpers import apply, session_fixture
from tests.integration.recording_helpers import take


@pytest.mark.parametrize(
    "suffix,mime",
    [("mp3", "audio/mpeg"), ("ogg", "audio/ogg"), ("m4a", "audio/mp4")],
)
def test_encoded_take_retains_original_bytes_and_receipt(client, package, suffix, mime):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    audio = (ROOT / f"fixtures/preservation/media/tone.{suffix}").read_bytes()
    body = take(
        result["session"]["attempts"][0],
        mime_type=mime,
        audio_base64=base64.b64encode(audio).decode(),
    )
    response = client.post("/api/v1/recordings", json=body)
    assert response.status_code == 200, response.text
    receipt = response.json()
    assert receipt["complete"]
    assert receipt["byte_length"] == len(audio)
    assert receipt["sha256"] == hashlib.sha256(audio).hexdigest()
    assert client.post("/api/v1/recordings", json=body).json() == receipt
    assert client.get(f"/api/v1/recordings/{receipt['id']}").content == audio
    retained = client.get(f"/api/v1/phrases/{body['target_id']}/recordings").json()
    assert retained == [receipt]


def test_opaque_take_is_stored_without_audio_inspection_or_capture_failure(client, package):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    capture, result = apply(client, result["session"], "respond", modality="recorded")
    original = b"opaque-browser-audio\x00\xff"
    body = take(
        result["session"]["attempts"][0],
        capture_event_id=capture["id"],
        mime_type="audio/mp4; codecs=mp4a.40.2",
        audio_base64=base64.b64encode(original).decode(),
    )
    response = client.post("/api/v1/recordings", json=body)
    assert response.status_code == 200, response.text
    receipt = response.json()
    assert receipt["complete"]
    assert receipt["byte_length"] == len(original)
    assert receipt["sha256"] == hashlib.sha256(original).hexdigest()
    assert client.get(f"/api/v1/recordings/{receipt['id']}").content == original
    assert client.post("/api/v1/recordings", json=body).json() == receipt
    unchanged = client.get(f"/api/v1/sessions/{session['id']}").json()
    assert unchanged == result["session"]
    assert not unchanged["attempts"][0]["technical_failure"]
    assert client.post("/api/v1/recordings", json={**body, "complete": False}).status_code == 409
    assert (
        client.post("/api/v1/recordings", json={**body, "mime_type": "audio/wav"}).status_code
        == 409
    )
    assert (
        client.post(
            "/api/v1/recordings", json={**body, "id": str(uuid4()), "mime_type": "text/plain"}
        ).status_code
        == 422
    )


def test_interrupted_bytes_remain_honestly_incomplete(client, package):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    original = b"unfinished-container-fragment"
    body = take(
        result["session"]["attempts"][0],
        complete=False,
        audio_base64=base64.b64encode(original).decode(),
    )
    response = client.post("/api/v1/recordings", json=body)
    assert response.status_code == 200, response.text
    receipt = response.json()
    assert not receipt["complete"]
    assert client.get(f"/api/v1/recordings/{receipt['id']}").content == original
