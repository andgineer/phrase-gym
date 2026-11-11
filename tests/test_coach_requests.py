import base64
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from phrase_gym.server.app import create_app
from phrase_gym.server.coach_requests import request_id
from tests.phrase_packages import coach_card, prepare_typed, publish, silent_reference
from tests.practice_client import Practice

TYPED = "strongly believe — We strongly believe this will help."


def submit(client: TestClient, text: str = TYPED) -> dict[str, str]:
    response = client.post("/api/coach/requests", json={"text": text})
    assert response.status_code == 200, response.text
    return response.json()


def result(package_id: str, **changes: object) -> dict[str, object]:
    return {
        "result_id": package_id,
        "phrase": coach_card(package_id, **changes),
        "reference": silent_reference(),
    }


def coach_lesson(client: TestClient) -> dict[str, object]:
    [item] = [
        lesson
        for lesson in client.get("/api/lessons").json()["items"]
        if lesson["title"] == "Added with Coach"
    ]
    return item


def test_typed_text_waits_offline_and_survives_restart(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    with TestClient(create_app()) as client:
        saved = submit(client, f"  {TYPED}  ")
        assert saved == {"request_id": request_id(TYPED), "outcome": "saved"}
        assert client.get(f"/api/imports/{saved['request_id']}").json()["status"] == "pending"
        assert client.post("/api/imports/coach/next").json()["text"] == TYPED
    with TestClient(create_app()) as client:
        work = client.post("/api/imports/coach/next").json()
        assert work == {"package_id": saved["request_id"], "text": TYPED}
        first = client.post(
            f"/api/imports/{work['package_id']}/result", json=result(saved["request_id"])
        )
        assert first.json() == {
            "package_id": saved["request_id"],
            "status": "added",
            "error": None,
            "added_count": 1,
        }
        again = client.post(
            f"/api/imports/{work['package_id']}/result", json=result(saved["request_id"])
        )
        assert again.json() == first.json()
        changed = result(saved["request_id"], explanation="Another explanation of the chunk.")
        assert (
            client.post(f"/api/imports/{work['package_id']}/result", json=changed).status_code
            == 409
        )
        assert client.post("/api/imports/coach/next").json() is None
        assert coach_lesson(client)["phrase_count"] == 1


def test_resubmitting_the_same_text_reports_its_state_without_duplicates(
    client: TestClient,
) -> None:
    package_id = submit(client)["request_id"]
    assert submit(client, TYPED.upper().replace(" ", "  "))["outcome"] == "waiting"
    client.post("/api/imports/coach/next")
    assert submit(client)["outcome"] == "waiting"
    client.post(f"/api/imports/{package_id}/result", json=result(package_id))
    assert submit(client) == {"request_id": package_id, "outcome": "in_library"}
    assert client.post("/api/imports/coach/next").json() is None
    assert coach_lesson(client)["phrase_count"] == 1


def test_deleted_card_is_prepared_again_when_its_text_is_resubmitted(client: TestClient) -> None:
    prepare_typed(client, TYPED)
    [phrase] = client.get("/api/phrases", params={"query": "strongly"}).json()["items"]
    assert client.delete(f"/api/phrases/{phrase['id']}").status_code == 200
    assert submit(client)["outcome"] == "saved"
    assert prepare_typed(client, TYPED)["added_count"] == 1
    assert coach_lesson(client)["phrase_count"] == 1


def test_typed_phrase_and_word_share_one_coach_lesson_apart_from_courses(
    client: TestClient, package: dict[str, object]
) -> None:
    publish(client, package)
    prepare_typed(client, TYPED)
    word_contexts = [
        {"situation": "She was __________ to sign the contract. (=unwilling)"},
        {"situation": "He seemed __________ to talk about it. (=not eager)"},
        {"situation": "I'm __________ to change the plan now. (=hesitant)"},
    ]
    prepare_typed(
        client,
        "reluctant",
        text="She was reluctant to leave.",
        explanation="Word: reluctant — unwilling and slow to do something.",
        cues=word_contexts,
        recall_hints=["relu... / to", "re... / -ant"],
    )
    titles = [lesson["title"] for lesson in client.get("/api/lessons").json()["items"]]
    assert titles == ["Meetings", "Added with Coach"]
    assert coach_lesson(client)["phrase_count"] == 2
    found = client.get("/api/phrases", params={"query": "reluctant"}).json()["items"]
    details = client.get(f"/api/phrases/{found[0]['id']}").json()
    assert details["source_label"] == "Added with Coach"
    practice = Practice(client)
    card = practice.card()
    assert card["cue"]


def test_invalid_results_never_become_phrases(client: TestClient) -> None:
    package_id = submit(client)["request_id"]
    client.post("/api/imports/coach/next")
    url = f"/api/imports/{package_id}/result"
    wrong_item = result(package_id)
    wrong_item["phrase"]["item_id"] = "other-item"  # type: ignore[index]
    assert client.post(url, json=wrong_item).status_code == 409
    bad_audio = result(package_id)
    bad_audio["reference"] = base64.b64encode(b"not wave").decode()
    response = client.post(url, json=bad_audio)
    assert response.status_code == 409
    assert "invalid reference audio" in response.text
    no_hints = result(package_id, recall_hints=["only one"])
    assert client.post(url, json=no_hints).status_code == 422
    unknown = result(package_id, target_chunk="strongly believe")
    assert client.post(url, json=unknown).status_code == 422
    assert client.get("/api/lessons").json()["all_lessons"]["phrase_count"] == 0
    assert client.get(f"/api/imports/{package_id}").json()["status"] == "preparing"


def test_blank_or_oversized_text_is_rejected(client: TestClient) -> None:
    assert client.post("/api/coach/requests", json={"text": "   "}).status_code == 422
    assert client.post("/api/coach/requests", json={"text": "x" * 1001}).status_code == 422
    assert client.post("/api/imports/coach/next").json() is None


def test_coach_routes_ignore_external_packages(
    client: TestClient, package: dict[str, object]
) -> None:
    publish(client, package)
    package_id = package["package_id"]
    assert client.post(f"/api/imports/{package_id}/failure", json={"error": "x"}).status_code == 404
    assert (
        client.post(f"/api/imports/{package_id}/result", json=result(str(package_id))).status_code
        == 404
    )
