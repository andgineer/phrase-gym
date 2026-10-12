import base64
from uuid import uuid4

from fastapi.testclient import TestClient

from phrase_gym.api.app import create_app
from tests.integration.helpers import NOW, wav_bytes
from tests.integration.practice_helpers import apply, session_fixture


def take(attempt, **changes):
    return {
        "id": str(uuid4()),
        "target_id": attempt["target_id"],
        "revision": attempt["revision"],
        "attempt_id": attempt["id"],
        "captured_at": NOW.isoformat(),
        "mime_type": "audio/wav",
        "duration_ms": 100,
        "complete": True,
        "audio_base64": base64.b64encode(wav_bytes()).decode(),
        **changes,
    }


def test_all_takes_retained_idempotently_after_completion_and_restart(client, package, config):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    attempt = result["session"]["attempts"][0]
    bodies = [
        take(attempt),
        take(attempt, complete=False, audio_base64=base64.b64encode(b"interrupted").decode()),
    ]
    ids = []
    for body in bodies:
        response = client.post("/api/v1/recordings", json=body)
        assert response.status_code == 200, response.text
        ids.append(response.json()["id"])
        assert client.post("/api/v1/recordings", json=body).json() == response.json()
    assert (
        client.post("/api/v1/recordings", json={**bodies[0], "complete": False}).status_code == 409
    )
    session = result["session"]
    for _ in range(3):
        if session["cursor"] > 0:
            _, result = apply(client, session, "present")
            session = result["session"]
        _, result = apply(client, session, "rate", rating="recalled")
        session = result["session"]
    with TestClient(create_app(config)) as restarted:
        retained = restarted.get(f"/api/v1/phrases/{attempt['target_id']}/recordings").json()
        assert {item["id"] for item in retained} == set(ids)
        assert restarted.get(f"/api/v1/recordings/{ids[0]}").content == wav_bytes()
        assert restarted.get(f"/api/v1/recordings/{ids[1]}").content == b"interrupted"


def test_lookup_take_requires_actual_exposure_without_a_session(client, package):
    session = session_fixture(client, package)
    phrase = session["manifest"]["targets"][0]["phrase"]
    exposure_id = str(uuid4())
    exposure = {
        "id": exposure_id,
        "target_id": phrase["id"],
        "revision": phrase["revision"],
        "language": "en",
        "kind": "explanation",
    }
    assert client.post("/api/v1/exposures", json=exposure).status_code == 200
    body = {
        "id": str(uuid4()),
        "target_id": phrase["id"],
        "revision": phrase["revision"],
        "exposure_id": exposure_id,
        "captured_at": NOW.isoformat(),
        "mime_type": "audio/wav",
        "complete": True,
        "audio_base64": base64.b64encode(wav_bytes()).decode(),
    }
    assert client.post("/api/v1/recordings", json=body).status_code == 200
    assert client.get("/api/v1/sessions/current").json()["id"] == session["id"]
    assert (
        client.post(
            "/api/v1/recordings", json={**body, "id": str(uuid4()), "exposure_id": str(uuid4())}
        ).status_code
        == 422
    )


def test_corrupt_complete_audio_bad_context_and_unknown_take_are_rejected(client, package):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    attempt = result["session"]["attempts"][0]
    assert (
        client.post("/api/v1/recordings", json=take(attempt, audio_base64="not base64")).status_code
        == 422
    )
    assert (
        client.post(
            "/api/v1/recordings", json=take(attempt, audio_base64="aW52YWxpZA==")
        ).status_code
        == 422
    )
    assert (
        client.post("/api/v1/recordings", json=take(attempt, attempt_id=str(uuid4()))).status_code
        == 409
    )
    assert client.post("/api/v1/recordings", json=take(attempt, revision=999)).status_code == 422
    assert client.get(f"/api/v1/recordings/{uuid4()}").status_code == 404


def test_recording_byte_ranges_and_capture_support_are_pinned(client, package):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    capture, result = apply(client, result["session"], "respond", modality="recorded")
    assert result["receipts"][0]["capture_supported"] is False
    attempt = result["session"]["attempts"][0]
    first = client.post("/api/v1/recordings", json=take(attempt, capture_event_id=capture["id"]))
    assert first.status_code == 200, first.text
    _, result = apply(client, result["session"], "respond", modality="recorded")
    assert result["receipts"][0]["capture_supported"] is True
    assert result["session"]["attempts"][0]["original"]["supported"] is False
    path = f"/api/v1/recordings/{first.json()['id']}"
    partial = client.get(path, headers={"Range": "bytes=0-9"})
    assert partial.status_code == 206 and partial.content == wav_bytes()[:10]
    assert partial.headers["content-range"] == f"bytes 0-9/{len(wav_bytes())}"
    assert client.get(path, headers={"Range": "bytes=-10"}).content == wav_bytes()[-10:]
    assert client.get(path, headers={"Range": "bytes=999999-"}).status_code == 416
