import base64
import time
from pathlib import Path
from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

from phrase_gym.app import create_app


def wait_added(client: TestClient, package_id: str) -> dict[str, object]:
    for _ in range(100):
        state = client.get(f"/api/imports/{package_id}").json()
        if state["status"] != "preparing":
            return state
        time.sleep(0.01)
    raise AssertionError("import remained preparing")


def test_study_recordings_and_restart(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response = client.post("/api/imports", json={"package": package})
    assert response.status_code == 202
    assert wait_added(client, str(package["package_id"]))["added_count"] == 5
    study = client.post("/api/study/current").json()
    assert study["studied"] == 0
    assert study["card"]["answer"] == "Complete phrase number 1."
    attempt_id = study["card"]["attempt_id"]
    card_id = study["card"]["id"]
    audio = b"final-encoder-tail"
    recording_id = str(uuid4())
    upload = {
        "id": recording_id,
        "card_id": card_id,
        "attempt_id": attempt_id,
        "captured_at": "2026-10-25T10:00:00Z",
        "mime_type": "audio/webm;codecs=opus",
        "duration_ms": 1200,
        "complete": True,
        "capture_settings": {"sampleRate": 48000},
        "audio_base64": base64.b64encode(audio).decode(),
    }
    receipt = client.post("/api/recordings", json=upload).json()
    assert receipt["byte_length"] == len(audio)
    assert client.post("/api/recordings", json=upload).json() == receipt
    completed = client.post(
        f"/api/attempts/{attempt_id}/complete", json={"operation_id": str(uuid4())}
    ).json()
    assert completed["graded"] is False

    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    monkeypatch.setenv("PHRASE_GYM_FAKE_TTS", "1")
    with TestClient(create_app()) as restarted:
        assert restarted.get(f"/api/recordings/{recording_id}/audio").content == audio
        next_card = restarted.post("/api/study/current").json()
        assert next_card["studied"] == 1
        assert next_card["card"]["item_id"] == "meeting-2"
        history = restarted.get("/api/study/history").json()
        assert history[0]["id"] == card_id
        assert history[0]["recordings"][0]["id"] == recording_id


def test_local_boundary_rejects_unrelated_websites(client: TestClient) -> None:
    response = client.post(
        "/api/study/current",
        headers={"origin": "https://attacker.example", "sec-fetch-site": "cross-site"},
    )
    assert response.status_code == 403
    assert client.get("/api/health", headers={"host": "evil.example"}).status_code == 400
