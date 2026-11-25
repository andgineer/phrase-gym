import base64
import sqlite3
from pathlib import Path
from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

from phrase_gym.server import publication
from tests.phrase_packages import coach_card, silent_reference
from tests.practice_client import Practice

SENTENCE = "We strongly believe this will work"
GAPPED = "We __________ this will work"
INSTRUCTION = "Say the missing wording."


def basic(**changes: object) -> dict[str, object]:
    return {
        "target": "strongly believe",
        "text": SENTENCE,
        "cue": GAPPED,
        "instruction": INSTRUCTION,
        **changes,
    }


def claimed(client: TestClient, text: str = SENTENCE) -> str:
    request_id = client.post(
        "/api/coach/requests", json={"text": text, "learning_language": "en"}
    ).json()["request_id"]
    assert client.post("/api/imports/coach/next").json()["package_id"] == request_id
    return request_id


def post_result(
    client: TestClient, request_id: str, card: dict[str, object], reference: str | None = None
):
    return client.post(
        f"/api/imports/{request_id}/result",
        json={"result_id": request_id, "basic": card, "reference": reference or silent_reference()},
    )


def coach_phrases(client: TestClient) -> list[dict[str, object]]:
    lessons = client.get("/api/lessons").json()["items"]
    return [
        phrase
        for lesson in lessons
        if lesson["title"] == "Added with Coach"
        for phrase in client.get("/api/phrases", params={"lesson_id": lesson["id"]}).json()["items"]
    ]


def reference_files(data_dir: Path) -> list[Path]:
    folder = data_dir / "references"
    return sorted(folder.iterdir()) if folder.exists() else []


def test_a_basic_card_practises_the_source_gap_and_reveals_the_sentence_with_its_audio(
    client: TestClient,
) -> None:
    request_id = claimed(client)
    reference = silent_reference()
    response = post_result(client, request_id, basic(), reference)
    assert response.json() == {
        "package_id": request_id,
        "status": "added",
        "error": None,
        "added_count": 1,
    }
    assert [phrase["text"] for phrase in coach_phrases(client)] == [SENTENCE]

    practice = Practice(client)
    card = practice.card()
    assert (card["instruction"], card["cue"]) == (
        INSTRUCTION,
        {"situation": GAPPED, "partner_line": None},
    )
    assert (card["hint_available"], card["other_cue_available"], card["answer"]) == (
        False,
        False,
        None,
    )
    assert practice.action("hint", card, operation_id=str(uuid4())).status_code == 409
    answer = practice.reveal(card)["answer"]
    assert answer["text"] == SENTENCE
    assert (answer["explanation"], answer["usage_contexts"]) == ("", [])
    audio = client.get(answer["reference_url"])
    assert audio.content == base64.b64decode(reference)

    details = client.get(f"/api/phrases/{card['phrase_id']}").json()
    assert (details["text"], details["explanation"]) == (SENTENCE, "")


def test_replaying_a_completed_result_creates_no_duplicate(client: TestClient) -> None:
    request_id = claimed(client)
    reference = silent_reference()
    first = post_result(client, request_id, basic(), reference)
    again = post_result(client, request_id, basic(), reference)
    assert again.json() == first.json()
    other = post_result(client, request_id, basic(cue="st____ly be___ve"), reference)
    assert other.status_code == 409
    assert len(coach_phrases(client)) == 1
    resubmitted = client.post(
        "/api/coach/requests", json={"text": SENTENCE, "learning_language": "en"}
    ).json()
    assert resubmitted == {"request_id": request_id, "outcome": "in_library"}


def test_failed_audio_publishes_no_card(client: TestClient, tmp_path: Path) -> None:
    request_id = claimed(client)
    bad = base64.b64encode(b"not a wave file").decode()
    response = post_result(client, request_id, basic(), bad)
    assert response.status_code == 409
    assert "invalid reference audio" in response.text
    assert coach_phrases(client) == []
    assert reference_files(tmp_path) == []
    assert client.get(f"/api/imports/{request_id}").json()["status"] == "preparing"


def test_a_failed_publication_leaves_no_partial_card(
    client: TestClient, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    request_id = claimed(client)

    def broken_insert(*_: object) -> None:
        raise sqlite3.OperationalError("disk I/O error")

    monkeypatch.setattr(publication, "_insert_phrase", broken_insert)
    with pytest.raises(sqlite3.OperationalError):
        post_result(client, request_id, basic())
    monkeypatch.undo()
    assert reference_files(tmp_path) == []
    assert client.get("/api/lessons").json()["all_lessons"]["phrase_count"] == 0
    assert client.get(f"/api/imports/{request_id}").json()["status"] == "preparing"
    assert post_result(client, request_id, basic()).json()["status"] == "added"


@pytest.mark.parametrize(
    "changes",
    [
        {"cue": "We strongly believe __________ will work"},
        {"instruction": "Say strongly believe."},
        {"text": "We think this will work"},
        {"text": "We strongly believe it will work"},
        {"correction": {"original": "strongly belive", "corrected": "strongly believed"}},
    ],
)
def test_a_basic_result_must_conceal_the_target_and_reveal_it(
    client: TestClient, changes: dict[str, object]
) -> None:
    request_id = claimed(client)
    assert post_result(client, request_id, basic(**changes)).status_code == 422
    assert coach_phrases(client) == []


def test_a_result_carries_exactly_one_card(client: TestClient) -> None:
    request_id = claimed(client)
    url = f"/api/imports/{request_id}/result"
    both = {
        "result_id": request_id,
        "basic": basic(),
        "phrase": coach_card(request_id),
        "reference": silent_reference(),
    }
    assert client.post(url, json=both).status_code == 422
    neither = {"result_id": request_id, "reference": silent_reference()}
    assert client.post(url, json=neither).status_code == 422
    assert coach_phrases(client) == []


def test_a_corrected_wording_is_identified_on_the_card_and_in_the_coach_inbox(
    client: TestClient,
) -> None:
    request_id = claimed(client, "definately")
    correction = {"original": "definately", "corrected": "definitely"}
    card = basic(target="definitely", text="definitely", cue="def…ly", correction=correction)
    assert post_result(client, request_id, card).json()["status"] == "added"
    practice = Practice(client)
    shown = practice.card()
    assert shown["cue"] == {"situation": "def…ly", "partner_line": None}
    answer = practice.reveal(shown)["answer"]
    assert (answer["text"], answer["explanation"]) == ("definitely", "Corrected from “definately”.")
    [item] = client.get("/api/coach/inbox").json()["items"]
    assert item == {
        "request_id": request_id,
        "text": "definately",
        "learning_language": "en",
        "outcome": "corrected",
        "reason": None,
        "correction": correction,
    }
    assert client.post(f"/api/coach/requests/{request_id}/dismiss").status_code == 200
    assert client.get("/api/coach/inbox").json()["items"] == []
