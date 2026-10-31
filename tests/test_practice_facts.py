from datetime import UTC, datetime
from typing import Any
from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

from tests.phrase_packages import Package, lesson, make_package, phrase, publish, upload_take
from tests.practice_client import Card, Practice


@pytest.fixture
def practice(client: TestClient) -> Practice:
    return Practice(client)


def many(count: int) -> Package:
    return make_package(lessons=[lesson("unit-1", "Meetings", [phrase(i) for i in range(count)])])


def facts(practice: Practice) -> dict[str, Any]:
    response = practice.client.get("/api/practice/facts")
    assert response.status_code == 200, response.text
    return response.json()


def studied(practice: Practice, day: str, offset: int = 0, limit: int = 10) -> dict[str, Any]:
    response = practice.client.get(
        "/api/practice/studied",
        params={"learning_date": day, "offset": offset, "limit": limit},
    )
    assert response.status_code == 200, response.text
    return response.json()


def logged(practice: Practice, *days: str) -> None:
    with practice.database.write() as connection:
        connection.executemany(
            "INSERT INTO practice_log (id, phrase_id, phrase_text, lesson_title, completed_at, "
            "learning_date, kind, rating, skipped, fsrs_updated, mode) "
            "VALUES (?, NULL, 'Earlier phrase.', 'Earlier', ?, ?, 'recall', 'got_it', 0, 1, "
            "'daily')",
            [(str(uuid4()), f"{day}T09:00:00+00:00", day) for day in days],
        )


def rate(practice: Practice, card: Card, outcome: str) -> dict[str, Any]:
    if outcome != "skipped":
        practice.reveal(card)
    return practice.finish(card, outcome)


def streak(practice: Practice) -> tuple[int, int, list[bool]]:
    found = facts(practice)["streak"]
    return found["current"], found["best"], [day["practiced"] for day in found["recent_days"]]


def test_fresh_library_has_no_facts_and_no_next_return(practice: Practice) -> None:
    assert facts(practice) == {
        "learning_date": "2026-10-28",
        "streak": {
            "current": 0,
            "best": 0,
            "recent_days": [
                {"learning_date": f"2026-10-{day}", "practiced": False} for day in range(22, 29)
            ],
        },
        "today": dict.fromkeys(
            [
                "completions",
                "first_meetings",
                "recalls",
                "skipped",
                "got_it",
                "effort",
                "not_yet",
                "recordings",
            ],
            0,
        ),
        "work": {"completions": 0, "first_meetings": 0, "practice_days": 0},
        "next_return": None,
    }


def test_today_counts_cover_daily_extra_and_practice_outcomes(practice: Practice) -> None:
    publish(practice.client, many(6))
    practice.study_all_new()
    practice.later(days=1)
    state = practice.daily()
    for outcome in ["got_it", "effort", "not_yet", "skipped", "got_it"]:
        state = rate(practice, state["card"], outcome)["state"]
    assert state["card"]["kind"] == "first_study"
    practice.finish(state["card"], "studied")
    extra = practice.extra()["state"]
    assert extra["card"]["kind"] == "recall"
    rate(practice, extra["card"], "got_it")
    phrase_id = practice.log()[0]["phrase_id"]
    rate(practice, practice.practise(phrase_id), "effort")
    assert facts(practice)["today"] == {
        "completions": 8,
        "first_meetings": 1,
        "recalls": 7,
        "skipped": 1,
        "got_it": 3,
        "effort": 2,
        "not_yet": 1,
        "recordings": 0,
    }
    assert facts(practice)["work"] == {"completions": 13, "first_meetings": 6, "practice_days": 2}
    modes = [item["mode"] for item in studied(practice, "2026-10-29")["items"]]
    assert modes == ["daily"] * 6 + ["extra", "practice"]


def test_today_list_pages_chronologically_with_outcome_and_snapshot(practice: Practice) -> None:
    publish(practice.client, many(5))
    practice.study_all_new()
    first = studied(practice, "2026-10-28", limit=2)
    assert [item["text"] for item in first["items"]] == [
        "Complete phrase number 0.",
        "Complete phrase number 1.",
    ]
    assert first["items"][0] | {"completion_id": "", "phrase_id": ""} == {
        "completion_id": "",
        "phrase_id": "",
        "text": "Complete phrase number 0.",
        "lesson_title": "Meetings",
        "mode": "daily",
        "kind": "first_study",
        "rating": None,
        "skipped": False,
    }
    assert first["next_offset"] == 2
    second = studied(practice, "2026-10-28", offset=2, limit=2)
    third = studied(practice, "2026-10-28", offset=4, limit=2)
    assert [item["text"][-2] for item in second["items"] + third["items"]] == ["2", "3", "4"]
    assert third["next_offset"] is None
    assert studied(practice, "2026-10-27")["items"] == []
    bad = practice.client.get("/api/practice/studied", params={"learning_date": "yesterday"})
    assert bad.status_code == 422


def test_deleted_phrase_keeps_its_completed_facts(practice: Practice) -> None:
    publish(practice.client, many(2))
    card = practice.card()
    upload_take(practice.client, card, "deleted-take-0001", captured_at=practice.now.isoformat())
    practice.finish(card, "studied")
    assert facts(practice)["today"]["recordings"] == 1
    assert practice.client.delete(f"/api/phrases/{card['phrase_id']}").status_code == 200
    found = facts(practice)
    assert found["today"]["completions"] == 1 and found["today"]["recordings"] == 0
    assert found["work"] == {"completions": 1, "first_meetings": 1, "practice_days": 1}
    assert found["streak"]["current"] == 1
    [item] = studied(practice, "2026-10-28")["items"]
    assert item["phrase_id"] is None and item["text"] == "Complete phrase number 0."
    assert item["lesson_title"] == "Meetings"


def test_extra_and_completion_retries_do_not_duplicate_counts(practice: Practice) -> None:
    publish(practice.client, many(8))
    practice.study_all_new()
    operation = str(uuid4())
    assert practice.extra(operation_id=operation)["state"]["card"]["kind"] == "first_study"
    assert practice.extra(operation_id=operation)["replayed"] is True
    practice.extra()
    card = practice.daily()["card"]
    completion = str(uuid4())
    practice.finish(card, "studied", completion)
    assert practice.finish(card, "studied", completion)["replayed"] is True
    today = facts(practice)["today"]
    assert (today["completions"], today["first_meetings"]) == (6, 6)
    assert len(studied(practice, "2026-10-28")["items"]) == 6


def test_live_limit_increase_continues_the_same_daily_entry(practice: Practice) -> None:
    publish(practice.client, many(4))
    practice.set_limits(20, 1)
    state = practice.finish(practice.card(), "studied")["state"]
    assert state["card"] is None and state["message"] == "Today's Daily practice is complete."
    practice.set_limits(20, 2)
    state = practice.finish(practice.card(), "studied")["state"]
    assert state["card"] is None
    assert facts(practice)["today"]["first_meetings"] == 2


def test_multiple_takes_count_once_saved_and_unsaved_takes_do_not(practice: Practice) -> None:
    publish(practice.client, many(2))
    card = practice.card()
    for index in range(3):
        response = upload_take(
            practice.client, card, f"daily-take-000{index}", captured_at="2026-10-28T11:00:00Z"
        )
        assert response.status_code == 200, response.text
    upload_take(practice.client, card, "daily-take-0000", captured_at="2026-10-28T11:00:00Z")
    upload_take(practice.client, card, "early-take-0001", captured_at="2026-10-27T22:59:59Z")
    upload_take(practice.client, card, "local-take-0001", captured_at="2026-10-28T08:30:00-05:00")
    upload_take(practice.client, card, "later-take-0001", captured_at="2026-10-28T23:00:00Z")
    found = facts(practice)
    # The Belgrade learning day spans 2026-10-27T23:00Z to 2026-10-28T23:00Z.
    assert found["today"]["recordings"] == 4
    assert found["today"]["completions"] == 0 and found["streak"]["current"] == 0


def test_streak_ends_yesterday_until_first_completion_and_resets_after_a_gap(
    practice: Practice,
) -> None:
    publish(practice.client, many(3))
    logged(practice, "2026-10-22", "2026-10-23", "2026-10-24", "2026-10-26", "2026-10-27")
    current, best, recent = streak(practice)
    assert (current, best) == (2, 3)
    assert recent == [True, True, True, False, True, True, False]
    practice.finish(practice.card(), "studied")
    assert streak(practice)[:2] == (3, 3)
    practice.later(days=1)
    assert streak(practice)[:2] == (3, 3)
    practice.later(days=1)
    current, best, recent = streak(practice)
    assert (current, best) == (0, 3)
    assert recent == [True, False, True, True, True, False, False]
    assert facts(practice)["work"]["practice_days"] == 6


def test_reveal_open_and_abandon_do_not_count_a_day(practice: Practice) -> None:
    publish(practice.client, many(2))
    practice.study_all_new()
    practice.later(days=1)
    card = practice.card()
    practice.choose("hint", card)
    practice.reveal(card)
    practice.daily()
    practice.practise(card["phrase_id"])
    found = facts(practice)
    assert found["today"]["completions"] == 0
    assert found["streak"]["current"] == 1 and not found["streak"]["recent_days"][-1]["practiced"]


def test_timezone_change_keeps_stored_learning_dates(practice: Practice) -> None:
    publish(practice.client, many(2))
    practice.now = datetime(2026, 10, 28, 22, 30, tzinfo=UTC)
    practice.finish(practice.card(), "studied")
    assert practice.log()[0]["learning_date"] == "2026-10-28"
    settings = practice.client.get("/api/settings").json()
    settings["learning_timezone"] = "Asia/Tokyo"
    assert practice.client.put("/api/settings", json=settings).status_code == 200
    found = facts(practice)
    assert found["learning_date"] == "2026-10-29"
    assert found["today"]["completions"] == 0
    assert found["streak"]["current"] == 1
    assert [item["text"] for item in studied(practice, "2026-10-28")["items"]] == [
        "Complete phrase number 0.",
    ]
    practice.finish(practice.card(), "studied")
    found = facts(practice)
    assert found["today"]["completions"] == 1 and found["streak"]["current"] == 2


def test_next_return_counts_phrases_due_by_the_next_learning_day(practice: Practice) -> None:
    publish(practice.client, many(4))
    assert facts(practice)["next_return"] is None
    practice.set_limits(1, 2)
    practice.study_all_new()
    assert facts(practice)["next_return"] == {"learning_date": "2026-10-29", "phrase_count": 2}
    practice.later(days=1)
    rate(practice, practice.card(), "got_it")
    # The phrase left beyond today's due allowance is still waiting today.
    assert facts(practice)["next_return"] == {"learning_date": "2026-10-29", "phrase_count": 1}
    with practice.database.write() as connection:
        connection.execute("UPDATE phrases SET due_at='2026-11-05T09:00:00+00:00'")
    found = facts(practice)["next_return"]
    assert found == {"learning_date": "2026-11-05", "phrase_count": 2}
