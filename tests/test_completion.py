import json
from pathlib import Path
from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

import phrase_gym.server.completion as completion
from phrase_gym.server.app import create_app
from tests.phrase_packages import Package, lesson, make_package, phrase, publish
from tests.practice_client import Card, Practice


@pytest.fixture
def practice(client: TestClient, package: Package) -> Practice:
    publish(client, package)
    return Practice(client)


def recall(practice: Practice) -> Card:
    practice.study_all_new()
    practice.later(days=1)
    card = practice.card()
    assert card["kind"] == "recall"
    return card


def test_recall_front_withholds_answer_and_a_rating_needs_reveal(practice: Practice) -> None:
    card = recall(practice)
    assert card["answer"] is None and card["recordings"] == [] and not card["revealed"]
    assert card["cue"]["situation"].startswith("Meeting situation number")
    assert practice.complete(card, "got_it").status_code == 409
    assert practice.complete(card, "studied").status_code == 409
    revealed = practice.reveal(card)
    assert revealed["answer"]["text"].startswith("Complete phrase number")
    assert revealed["view_token"] == card["view_token"]
    assert practice.log()[-1]["kind"] == "first_study"


def test_first_grade_updates_fsrs_due_and_last_review_together(practice: Practice) -> None:
    card = recall(practice)
    before = practice.phrase(card["phrase_id"])
    practice.reveal(card)
    receipt = practice.finish(card, "effort")["receipt"]
    assert receipt["rating"] == "effort" and receipt["fsrs_updated"] is True
    after = practice.phrase(card["phrase_id"])
    assert after["last_review"] == practice.now.isoformat()
    assert after["fsrs_json"] and json.loads(after["fsrs_json"])["due"] == after["due_at"]
    assert after["due_at"] > before["due_at"] and after["due_at"] >= "2026-10-29T23:00:00+00:00"
    assert after["studied_at"] == before["studied_at"]
    row = practice.log()[-1]
    assert (row["kind"], row["rating"], row["skipped"], row["fsrs_updated"], row["mode"]) == (
        "recall",
        "effort",
        0,
        1,
        "daily",
    )


def test_repeated_same_day_grade_records_practice_without_fsrs(practice: Practice) -> None:
    card = recall(practice)
    earlier = practice.now.replace(hour=8).isoformat()
    with practice.database.write() as connection:
        connection.execute(
            "UPDATE phrases SET last_review=? WHERE id=?", (earlier, card["phrase_id"])
        )
    before = practice.phrase(card["phrase_id"])
    practice.reveal(card)
    receipt = practice.finish(card, "not_yet")["receipt"]
    assert receipt["rating"] == "not_yet" and receipt["fsrs_updated"] is False
    assert practice.phrase(card["phrase_id"]) == before
    assert practice.log()[-1]["fsrs_updated"] == 0


def test_failed_save_rolls_back_and_the_same_id_retries(
    practice: Practice, monkeypatch: pytest.MonkeyPatch
) -> None:
    card = recall(practice)
    practice.reveal(card)
    before = practice.phrase(card["phrase_id"])
    log_before = practice.log()
    original = completion.advance

    def fail(*args: object) -> None:
        raise RuntimeError("disk full")

    monkeypatch.setattr(completion, "advance", fail)
    completion_id = str(uuid4())
    with pytest.raises(RuntimeError, match="disk full"):
        practice.complete(card, "got_it", completion_id)
    assert practice.phrase(card["phrase_id"]) == before
    assert practice.log() == log_before
    assert practice.rows("SELECT * FROM operations WHERE id=?", completion_id) == []
    assert practice.daily()["card"]["view_token"] == card["view_token"]
    monkeypatch.setattr(completion, "advance", original)
    result = practice.finish(card, "got_it", completion_id)
    assert result["replayed"] is False and result["receipt"]["fsrs_updated"] is True
    assert len(practice.log()) == len(log_before) + 1


def test_open_reveal_cue_hint_and_abandon_write_no_log_rows(practice: Practice) -> None:
    card = recall(practice)
    rows = len(practice.log())
    practice.choose("cue", card)
    practice.choose("hint", card)
    practice.reveal(card)
    assert practice.card() == practice.reveal(card)
    practice.later(days=1)
    reopened = practice.card()
    assert reopened["view_token"] != card["view_token"]
    assert practice.action("reveal", card).status_code == 409
    assert len(practice.log()) == rows


def test_view_tokens_are_fresh_for_every_appearance(practice: Practice) -> None:
    tokens = []
    state = practice.daily()
    resumed = practice.daily()["card"]
    assert resumed["view_token"] == state["card"]["view_token"]
    first = state["card"]
    while state["card"]:
        tokens.append(state["card"]["view_token"])
        state = practice.finish(state["card"], "studied")["state"]
    assert len(tokens) == len(set(tokens)) == 5
    practice.later(days=1)
    again = practice.card()
    assert again["phrase_id"] == first["phrase_id"]
    assert again["view_token"] not in tokens


def test_stale_tab_actions_conflict_even_with_new_ids(practice: Practice) -> None:
    old = practice.card()
    practice.finish(old, "studied")
    current = practice.card()
    rows = practice.log()
    assert practice.complete(old, "studied").status_code == 409
    assert practice.action("reveal", old).status_code == 409
    for path in ("cue", "hint"):
        assert practice.action(path, old, operation_id=str(uuid4())).status_code == 409
    mismatched = {**current, "phrase_id": old["phrase_id"]}
    assert practice.complete(mismatched, "studied").status_code == 409
    assert practice.log() == rows
    assert practice.card() == current


def test_exact_retry_returns_the_original_receipt_after_advances_and_restart(
    practice: Practice, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    first = practice.card()
    completion_id = str(uuid4())
    original = practice.finish(first, "studied", completion_id)
    state = original["state"]
    for _ in range(2):
        state = practice.finish(state["card"], "studied")["state"]
    current = practice.card()
    replay = practice.finish(first, "studied", completion_id)
    assert replay == {"receipt": original["receipt"], "replayed": True, "state": None}
    assert practice.card() == current
    assert len(practice.log()) == 3
    with TestClient(create_app()) as restarted:
        retried = Practice(restarted, practice.now)
        assert retried.finish(first, "studied", completion_id)["receipt"] == original["receipt"]
        assert retried.complete(first, "skipped", completion_id).status_code == 409
        assert retried.complete(current, "studied", completion_id).status_code == 409
        assert len(retried.log()) == 3


def test_deleting_the_open_phrase_invalidates_its_view_without_a_log_row(
    client: TestClient,
) -> None:
    publish(client, make_package(lessons=[lesson("unit", "Unit", [phrase(1), phrase(2)])]))
    practice = Practice(client)
    first = practice.card()
    practice.finish(first, "studied")
    open_card = practice.card()
    assert client.delete(f"/api/phrases/{open_card['phrase_id']}").status_code == 200
    assert practice.complete(open_card, "studied").status_code == 409
    assert practice.daily()["card"] is None
    assert len(practice.log()) == 1
    assert client.delete(f"/api/phrases/{first['phrase_id']}").status_code == 200
    kept = practice.log()[0]
    assert kept["phrase_id"] is None
    assert (kept["phrase_text"], kept["lesson_title"]) == ("Complete phrase number 1.", "Unit")
