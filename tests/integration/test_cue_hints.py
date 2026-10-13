import json

import pytest

from phrase_gym.storage import Database
from phrase_gym.tables import TargetRevision
from tests.integration.helpers import phrases
from tests.integration.session_helpers import ready_package, start


@pytest.mark.parametrize("visible", [[], ["punctuation"], ["whitespace"]])
def test_import_rejects_cues_without_phrase_specific_text(client, package, visible):
    ready_package(client, package)
    previous = phrases(client)
    target = package["targets"][0]
    target["chunks"] = [
        {"id": "opening", "text": "How has"},
        {"id": "whitespace", "text": " "},
        {"id": "subject", "text": "your day "},
        {"id": "ending", "text": "been so far"},
        {"id": "punctuation", "text": "?"},
    ]
    target["cues"][0]["visible_chunks"] = visible
    report = client.post("/api/v1/imports", json=package)
    assert report.status_code == 200
    assert report.json()["status"] == "rejected"
    assert "visible phrase-specific text" in report.json()["errors"][0]
    assert phrases(client) == previous


def test_legacy_situation_only_cue_stays_readable_but_cannot_enter_new_session(
    client, package, config
):
    content = ready_package(client, package)
    original = start(client).json()
    target = next(phrase for phrase in content if phrase["content"]["source_id"] == "day")
    database = Database(config.data_dir)
    try:
        with database.write() as transaction:
            revision = transaction.get(TargetRevision, (target["id"], target["revision"]))
            payload = json.loads(revision.payload)
            payload["cues"][0]["visible_chunks"] = []
            revision.payload = json.dumps(payload)
        legacy = client.get(f"/api/v1/phrases/{target['id']}?language=en").json()
        assert legacy["retrieval_ready"] is False
        assert legacy["audio_status"] == "ready"
        assert client.get(f"/api/v1/sessions/{original['id']}").json() == original
        readiness = client.get("/api/v1/sessions/readiness?language=en").json()
        assert readiness["preparing_count"] == 1 and readiness["new_count"] == 2
        replacement = start(client, takeover_session_id=original["id"]).json()
        assert target["id"] not in {
            item["phrase"]["id"] for item in replacement["manifest"]["targets"]
        }
    finally:
        database.close()
