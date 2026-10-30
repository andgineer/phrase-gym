from datetime import UTC, datetime

import pytest
from fastapi.testclient import TestClient

from tests.phrase_packages import Package, lesson, make_package, phrase, publish
from tests.practice_client import Practice


@pytest.fixture
def practice(client: TestClient) -> Practice:
    return Practice(client)


def many(count: int) -> Package:
    return make_package(lessons=[lesson("unit-1", "Meetings", [phrase(i) for i in range(count)])])


def test_empty_library_offers_no_card_and_page_load_starts_nothing(practice: Practice) -> None:
    state = practice.daily()
    assert state["card"] is None
    assert state["message"] == "Add your first phrase batch to begin."
    assert "/api/study/current" not in practice.client.get("/openapi.json").json()["paths"]


def test_first_study_shows_answer_and_next_schedules_tomorrow(
    practice: Practice, package: Package
) -> None:
    publish(practice.client, package)
    card = practice.card()
    assert card["kind"] == "first_study" and card["mode"] == "daily" and card["revealed"]
    assert card["cue"] is None and card["hint"] is None
    assert card["answer"]["text"] == "Complete phrase number 1."
    assert card["answer"]["reference_url"].endswith("/reference")
    result = practice.finish(card, "studied")
    assert result["receipt"] | {"completion_id": ""} == {
        "completion_id": "",
        "phrase_id": card["phrase_id"],
        "learning_date": "2026-10-28",
        "mode": "daily",
        "kind": "first_study",
        "rating": None,
        "skipped": False,
        "fsrs_updated": False,
    }
    studied = practice.phrase(card["phrase_id"])
    assert studied["studied_at"] == practice.now.isoformat()
    assert studied["due_at"] == "2026-10-28T23:00:00+00:00"
    assert studied["last_review"] is None and studied["fsrs_json"] is None
    assert result["state"]["card"]["phrase_id"] != card["phrase_id"]
    assert result["state"]["progress"]["new_completed"] == 1


def test_due_and_new_limits_are_independent(practice: Practice) -> None:
    publish(practice.client, many(30))
    ids = [row["id"] for row in practice.rows("SELECT id FROM phrases ORDER BY position")]
    practice.make_due(*ids[:25])
    kinds = []
    state = practice.daily()
    while state["card"]:
        card = state["card"]
        kinds.append(card["kind"])
        if card["kind"] == "first_study":
            state = practice.finish(card, "studied")["state"]
        elif len(kinds) == 3:
            state = practice.finish(card, "skipped")["state"]
        else:
            practice.reveal(card)
            state = practice.finish(card, "got_it")["state"]
    assert kinds == ["recall"] * 20 + ["first_study"] * 5
    assert state["progress"] == {
        "due_completed": 20,
        "due_allowance": 20,
        "new_completed": 5,
        "new_allowance": 5,
    }
    assert state["message"] == "Today's Daily practice is complete."
    assert len([row for row in practice.log() if row["skipped"]]) == 1


def test_exhausted_due_allowance_still_offers_new_phrases(practice: Practice) -> None:
    publish(practice.client, many(4))
    ids = [row["id"] for row in practice.rows("SELECT id FROM phrases ORDER BY position")]
    practice.make_due(*ids[:2])
    practice.set_limits(due=1, new=5)
    first = practice.card()
    assert first["kind"] == "recall"
    second = practice.finish(first, "skipped")["state"]["card"]
    assert second["kind"] == "first_study" and second["phrase_id"] == ids[2]


def test_live_limit_increase_after_apparent_exhaustion(practice: Practice) -> None:
    publish(practice.client, many(8))
    practice.study_all_new()
    done = practice.daily()
    assert done["card"] is None and done["message"] == "Today's Daily practice is complete."
    practice.set_limits(due=20, new=7)
    more = practice.daily()
    assert more["card"]["kind"] == "first_study"
    assert more["progress"]["new_allowance"] == 7
    practice.study_all_new()
    assert len(practice.log()) == 7
    practice.set_limits(due=20, new=3)
    assert practice.daily()["card"] is None


def test_stored_extra_target_extends_but_zero_setting_disables(practice: Practice) -> None:
    publish(practice.client, many(9))
    with practice.database.write() as connection:
        connection.execute("INSERT INTO extra_targets VALUES ('2026-10-28', 0, 7)")
    practice.study_all_new()
    assert len(practice.log()) == 7
    practice.set_limits(due=20, new=0)
    state = practice.daily()
    assert state["card"] is None and state["progress"]["new_allowance"] == 0


@pytest.mark.parametrize(
    "limits", [{"daily_due_limit": 0, "daily_new_limit": 0}, {"daily_due_limit": -1}]
)
def test_invalid_daily_limits_are_rejected(practice: Practice, limits: dict[str, int]) -> None:
    settings = practice.client.get("/api/settings").json()
    assert settings["daily_due_limit"] == 20 and settings["daily_new_limit"] == 5
    assert practice.client.put("/api/settings", json=settings | limits).status_code == 422
    assert practice.client.get("/api/settings").json() == settings


def test_skip_leaves_due_and_excludes_phrase_until_a_new_traversal(practice: Practice) -> None:
    publish(practice.client, many(2))
    ids = [row["id"] for row in practice.rows("SELECT id FROM phrases ORDER BY position")]
    practice.make_due(*ids)
    first = practice.card()
    before = practice.phrase(first["phrase_id"])
    second = practice.finish(first, "skipped")["state"]["card"]
    assert second["phrase_id"] != first["phrase_id"]
    assert practice.phrase(first["phrase_id"]) == before
    practice.reveal(second)
    assert practice.finish(second, "got_it")["state"]["card"] is None
    assert practice.daily()["message"] == "Today's Daily practice is complete."
    practice.later(days=1)
    assert practice.card()["phrase_id"] == first["phrase_id"]


def test_lesson_choice_affects_the_next_view_not_the_current_phrase(practice: Practice) -> None:
    publish(
        practice.client,
        make_package(
            lessons=[
                lesson("first", "First", [phrase(1), phrase(2)]),
                lesson("second", "Second", [phrase(3)]),
            ]
        ),
    )
    lessons = {
        item["title"]: item["id"] for item in practice.client.get("/api/lessons").json()["items"]
    }
    current = practice.card(lessons["First"])
    resumed = practice.card(lessons["Second"])
    assert resumed == current
    following = practice.finish(current, "studied")["state"]["card"]
    assert practice.phrase(following["phrase_id"])["item_id"] == "meeting-3"
    after = practice.finish(following, "studied")["state"]
    assert after["card"] is None
    assert after["message"] == "Today's Daily practice is complete. Next due: 2026-10-29."
    assert (
        practice.client.post(
            "/api/practice/daily", json={"lesson_id": "missing-lesson"}
        ).status_code
        == 404
    )


def test_nothing_waiting_and_empty_lesson_messages(practice: Practice) -> None:
    publish(practice.client, make_package(lessons=[lesson("only", "Only", [phrase(1)])]))
    lesson_id = practice.client.get("/api/lessons").json()["items"][0]["id"]
    practice.study_all_new()
    practice.later(days=1)
    waiting = practice.card()
    practice.reveal(waiting)
    practice.finish(waiting, "got_it")
    practice.later(days=1)
    with practice.database.write() as connection:
        connection.execute("UPDATE phrases SET due_at='2026-11-03T09:00:00+00:00'")
    state = practice.daily(lesson_id)
    assert state["message"] == "Nothing waiting today. Next due: 2026-11-03."
    practice.client.delete(f"/api/phrases/{waiting['phrase_id']}")
    assert practice.daily(lesson_id)["message"] == "No phrases in this lesson."


def test_timezone_change_closes_the_old_view_and_keeps_first_recall_later(
    practice: Practice, package: Package
) -> None:
    package["lessons"] = [lesson("unit-1", "Meetings", [phrase(1)])]
    publish(practice.client, package)
    practice.now = datetime(2026, 10, 28, 22, 30, tzinfo=UTC)
    practice.finish(practice.card(), "studied")
    assert practice.log()[0]["learning_date"] == "2026-10-28"
    practice.now = datetime(2026, 10, 28, 23, 5, tzinfo=UTC)
    belgrade = practice.card()
    assert belgrade["kind"] == "recall"
    settings = practice.client.get("/api/settings").json()
    settings["learning_timezone"] = "Europe/London"
    assert practice.client.put("/api/settings", json=settings).status_code == 200
    assert practice.action("reveal", belgrade).status_code == 409
    london = practice.daily()
    assert london["card"] is None
    assert london["progress"]["new_completed"] == 1
    assert london["message"] == "Today's Daily practice is complete. Next due: 2026-10-29."
    practice.now = datetime(2026, 10, 29, 0, 5, tzinfo=UTC)
    later = practice.card()
    assert later["kind"] == "recall" and later["view_token"] != belgrade["view_token"]
    assert practice.daily()["progress"]["new_completed"] == 0
