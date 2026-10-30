from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

from tests.phrase_packages import Package, lesson, phrase, publish, upload_take
from tests.practice_client import Practice

CUES = [
    {"situation": "Colleague asks: How's it going?", "partner_line": None},
    {"situation": "Friend asks: How's the new job?", "partner_line": "So, how is it?"},
]
HINTS = ["far", "so … good", "up to now"]


@pytest.fixture
def practice(client: TestClient, package: Package) -> Practice:
    cues = [{key: value for key, value in cue.items() if value} for cue in CUES]
    item = phrase(1, text="So far, so good.", cues=cues, recall_hints=HINTS)
    package["lessons"] = [lesson("small-talk", "Small talk", [item])]
    publish(client, package)
    practice = Practice(client)
    practice.study_all_new()
    practice.later(days=1)
    return practice


def test_new_cue_and_hint_choose_other_values_and_exact_retries_repeat(
    practice: Practice,
) -> None:
    card = practice.card()
    assert card["cue"] in CUES and card["other_cue_available"] and card["hint"] is None
    operation = str(uuid4())
    changed = practice.choose("cue", card, operation)
    assert changed["cue"] in CUES and changed["cue"] != card["cue"]
    assert changed["answer"] is None
    assert practice.choose("cue", card, operation)["cue"] == changed["cue"]
    first_hint = practice.choose("hint", card)
    hint_operation = str(uuid4())
    second_hint = practice.choose("hint", card, hint_operation)
    assert {first_hint["hint"], second_hint["hint"]} <= set(HINTS)
    assert second_hint["hint"] != first_hint["hint"]
    assert second_hint["cue"] == changed["cue"]
    later = practice.choose("hint", card)
    assert practice.choose("hint", card, hint_operation)["hint"] == second_hint["hint"]
    assert practice.card()["hint"] == later["hint"]
    assert practice.action("hint", card, operation_id=operation).status_code == 409
    assert practice.log()[-1]["kind"] == "first_study"


def test_next_appearance_avoids_the_last_shown_cue(practice: Practice) -> None:
    card = practice.card()
    shown = practice.choose("cue", card)["cue"]
    practice.reveal(card)
    practice.finish(card, "got_it")
    practice.later(days=30)
    following = practice.card()
    assert following["cue"] != shown


def test_first_study_offers_no_cue_or_hint(client: TestClient, package: Package) -> None:
    publish(client, package)
    card = Practice(client).card()
    assert card["kind"] == "first_study"
    for path in ("cue", "hint"):
        response = client.post(
            f"/api/practice/{path}",
            json={
                "view_token": card["view_token"],
                "phrase_id": card["phrase_id"],
                "operation_id": str(uuid4()),
            },
        )
        assert response.status_code == 409


def test_takes_keep_their_support_snapshots_and_upload_after_the_view_moves_on(
    practice: Practice,
) -> None:
    client = practice.client
    card = practice.choose("hint", practice.card())
    front_take = str(uuid4())
    assert upload_take(client, card, front_take).status_code == 200
    assert (
        upload_take(client, card, str(uuid4()), cue={"situation": "Invented cue."}).status_code
        == 409
    )
    assert upload_take(client, card, str(uuid4()), hint="invented").status_code == 409
    revealed = practice.reveal(card)
    assert [take["id"] for take in revealed["recordings"]] == [front_take]
    practice.finish(card, "got_it")
    progress = practice.daily()["progress"]
    log = practice.log()
    late_take = str(uuid4())
    assert upload_take(client, card, late_take, b"late tail").status_code == 200
    assert practice.daily()["progress"] == progress and practice.log() == log
    history = client.get(f"/api/phrases/{card['phrase_id']}/recordings").json()["items"]
    late = next(item for item in history if item["id"] == late_take)
    snapshot = {key: late[key] for key in ("mode", "kind", "cue", "hint", "answer")}
    assert snapshot == {
        "mode": "daily",
        "kind": "recall",
        "cue": card["cue"],
        "hint": card["hint"],
        "answer": "So far, so good.",
    }
    unknown = {**card, "phrase_id": "unknown-phrase-1"}
    assert upload_take(client, unknown, str(uuid4())).status_code == 409
