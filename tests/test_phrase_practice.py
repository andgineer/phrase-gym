from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

from tests.phrase_packages import lesson, make_package, phrase, publish, upload_take
from tests.practice_client import Practice


@pytest.fixture
def practice(client: TestClient) -> Practice:
    publish(
        client,
        make_package(lessons=[lesson("unit-1", "Meetings", [phrase(i) for i in range(1, 5)])]),
    )
    return Practice(client)


def ids(practice: Practice) -> list[str]:
    return [row["id"] for row in practice.rows("SELECT id FROM phrases ORDER BY position")]


def test_not_due_practice_grade_reschedules_once_per_learning_day(practice: Practice) -> None:
    practice.study_all_new()
    phrase_id = practice.log()[0]["phrase_id"]
    before = practice.phrase(phrase_id)
    progress = practice.daily()["progress"]
    card = practice.practise(phrase_id)
    assert card["kind"] == "recall" and card["answer"] is None and card["cue"]
    practice.reveal(card)
    result = practice.finish(card, "got_it")
    assert result["receipt"]["mode"] == "practice" and result["receipt"]["fsrs_updated"]
    assert result["state"] == {"card": None, "progress": progress, "message": None}
    after = practice.phrase(phrase_id)
    assert after["last_review"] == practice.now.isoformat() and after["due_at"] > before["due_at"]
    again = practice.practise(phrase_id)
    practice.reveal(again)
    assert practice.finish(again, "not_yet")["receipt"]["fsrs_updated"] is False
    assert practice.phrase(phrase_id) == after
    assert practice.daily()["progress"] == progress


def test_due_practice_grade_consumes_no_allowance_and_daily_skips_it_after(
    practice: Practice,
) -> None:
    first, second, *_ = ids(practice)
    practice.make_due(first, second)
    practice.set_limits(due=1, new=5)
    card = practice.practise(second)
    practice.reveal(card)
    assert practice.finish(card, "effort")["receipt"]["fsrs_updated"] is True
    daily = practice.daily()
    assert daily["progress"]["due_completed"] == 0
    assert daily["card"]["phrase_id"] == first and daily["card"]["mode"] == "daily"
    rows = practice.log()
    assert [(row["mode"], row["rating"]) for row in rows] == [("practice", "effort")]


def test_new_phrase_practice_is_a_first_meeting_completed_with_next(practice: Practice) -> None:
    phrase_id = ids(practice)[2]
    card = practice.practise(phrase_id)
    assert card["kind"] == "first_study" and card["answer"]["text"] == "Complete phrase number 3."
    assert practice.complete(card, "got_it").status_code == 409
    receipt = practice.finish(card, "studied")["receipt"]
    assert (receipt["kind"], receipt["mode"], receipt["fsrs_updated"]) == (
        "first_study",
        "practice",
        False,
    )
    studied = practice.phrase(phrase_id)
    assert studied["due_at"] == "2026-10-28T23:00:00+00:00" and studied["last_review"] is None
    daily = practice.daily()
    assert daily["progress"]["new_completed"] == 0
    assert daily["card"]["phrase_id"] != phrase_id


@pytest.mark.parametrize("same_phrase", [True, False])
def test_practice_closes_an_unfinished_daily_first_study_view(
    practice: Practice, same_phrase: bool
) -> None:
    daily = practice.card()
    assert daily["kind"] == "first_study"
    target = daily["phrase_id"] if same_phrase else ids(practice)[3]
    card = practice.practise(target)
    assert card["view_token"] != daily["view_token"]
    assert practice.complete(daily, "studied").status_code == 409
    assert practice.action("reveal", daily).status_code == 409
    assert practice.log() == []
    assert practice.phrase(daily["phrase_id"])["studied_at"] is None
    resumed = practice.daily()
    assert resumed["progress"]["new_completed"] == 0
    assert resumed["card"]["view_token"] not in (daily["view_token"], card["view_token"])
    assert resumed["card"]["mode"] == "daily" and resumed["card"]["phrase_id"] == daily["phrase_id"]
    assert practice.complete(card, "studied").status_code == 409
    assert practice.log() == []


def test_practice_keeps_the_regular_traversal_skips_and_its_own_skip_is_separate(
    practice: Practice,
) -> None:
    first, second, third, _ = ids(practice)
    practice.make_due(first, second, third)
    skipped = practice.card()
    assert skipped["phrase_id"] == first
    following = practice.finish(skipped, "skipped")["state"]["card"]
    assert following["phrase_id"] == second
    card = practice.practise(second)
    receipt = practice.finish(card, "skipped")["receipt"]
    assert (receipt["mode"], receipt["skipped"]) == ("practice", True)
    resumed = practice.daily()
    assert resumed["card"]["phrase_id"] == second
    assert resumed["card"]["view_token"] != following["view_token"]
    assert resumed["progress"]["due_completed"] == 1


def test_daily_closes_an_open_practice_view_and_unknown_phrases_are_not_found(
    practice: Practice,
) -> None:
    card = practice.practise(ids(practice)[1])
    daily = practice.card()
    assert daily["mode"] == "daily" and daily["view_token"] != card["view_token"]
    assert practice.complete(card, "studied").status_code == 409
    response = practice.client.post("/api/practice/phrase", json={"phrase_id": "missing-phrase"})
    assert response.status_code == 404
    assert practice.card() == daily


def test_practice_takes_and_late_uploads_keep_their_mode(practice: Practice) -> None:
    card = practice.practise(ids(practice)[0])
    early, late = str(uuid4()), str(uuid4())
    assert upload_take(practice.client, card, early).status_code == 200
    practice.finish(card, "studied")
    progress = practice.daily()["progress"]
    log = practice.log()
    assert upload_take(practice.client, card, late).status_code == 200
    assert practice.daily()["progress"] == progress and practice.log() == log
    history = practice.client.get(f"/api/phrases/{card['phrase_id']}/recordings").json()
    assert {(item["id"], item["mode"]) for item in history["items"]} == {
        (early, "practice"),
        (late, "practice"),
    }
