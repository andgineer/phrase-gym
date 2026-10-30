import base64
import hashlib
import os
import stat
from pathlib import Path
from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

from phrase_gym.server.app import create_app
from phrase_gym.server.storage import Database
from tests.phrase_packages import Package, prepare_with_coach
from tests.practice_client import Practice


def test_study_recordings_and_restart(
    client: TestClient,
    package: Package,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    assert prepare_with_coach(client, package)["added_count"] == 5
    if os.name != "nt":
        assert stat.S_IMODE(tmp_path.stat().st_mode) == 0o700
        assert stat.S_IMODE((tmp_path / "phrase-gym.sqlite3").stat().st_mode) == 0o600
        reference = next((tmp_path / "references").iterdir())
        assert stat.S_IMODE(reference.stat().st_mode) == 0o600
    practice = Practice(client)
    card = practice.card()
    assert card["answer"]["text"] == "Complete phrase number 1."
    phrase_id = card["phrase_id"]
    audio = b"final-encoder-tail"
    recording_id = str(uuid4())
    upload = {
        "id": recording_id,
        "phrase_id": phrase_id,
        "view_token": card["view_token"],
        "mode": "daily",
        "kind": "first_study",
        "captured_at": "2026-10-25T10:00:00Z",
        "mime_type": "audio/webm;codecs=opus",
        "duration_ms": 1200,
        "complete": True,
        "capture_settings": {"sampleRate": 48000},
        "audio_base64": base64.b64encode(audio).decode(),
    }
    pending = tmp_path / "recordings" / phrase_id / f"{recording_id}.pending"
    pending.parent.mkdir(parents=True)
    pending.write_bytes(b"partial-before-crash")
    receipt = client.post("/api/recordings", json=upload).json()
    assert receipt["byte_length"] == len(audio)
    assert receipt["sha256"] == hashlib.sha256(audio).hexdigest()
    assert client.post("/api/recordings", json=upload).json() == receipt
    if os.name != "nt":
        saved = tmp_path / "recordings" / phrase_id / recording_id
        assert stat.S_IMODE(saved.parent.stat().st_mode) == 0o700
        assert stat.S_IMODE(saved.stat().st_mode) == 0o600
    completed = practice.finish(card, "studied")
    assert completed["receipt"]["fsrs_updated"] is False

    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    monkeypatch.setenv("PHRASE_GYM_FAKE_TTS", "1")
    with TestClient(create_app()) as restarted:
        assert restarted.get(f"/api/recordings/{recording_id}/audio").content == audio
        next_card = Practice(restarted, practice.now).card()
        assert next_card["answer"]["text"] == "Complete phrase number 2."
        history = restarted.get(f"/api/phrases/{phrase_id}/recordings").json()
        assert [item["id"] for item in history["items"]] == [recording_id]
        assert history["items"][0]["mode"] == "daily"
        assert history["items"][0]["kind"] == "first_study"
        assert history["items"][0]["answer"] == "Complete phrase number 1."
        assert history["items"][0]["cue"] is None


def test_local_boundary_rejects_unrelated_websites(client: TestClient) -> None:
    response = client.post(
        "/api/practice/daily",
        json={},
        headers={"origin": "https://attacker.example", "sec-fetch-site": "cross-site"},
    )
    assert response.status_code == 403
    assert client.get("/api/health", headers={"host": "evil.example"}).status_code == 400


def test_study_contracts_are_closed_generated_schemas(client: TestClient) -> None:
    schemas = client.get("/openapi.json").json()["components"]["schemas"]
    for name in (
        "PracticeState",
        "PracticeCard",
        "CompletionResult",
        "CompletionRequest",
        "LessonPage",
        "PhrasePage",
        "PhraseDetails",
        "RecordingPage",
        "DeletionResult",
    ):
        assert schemas[name]["additionalProperties"] is False


@pytest.mark.skipif(os.name == "nt", reason="POSIX permission modes only")
def test_database_sidecars_are_private_while_open(tmp_path: Path) -> None:
    database = Database(tmp_path)
    with database.write() as connection:
        connection.execute("INSERT OR REPLACE INTO meta VALUES ('privacy-check', '1')")
        for suffix in ("", "-wal", "-shm"):
            path = Path(f"{database.path}{suffix}")
            assert path.exists()
            assert stat.S_IMODE(path.stat().st_mode) == 0o600
