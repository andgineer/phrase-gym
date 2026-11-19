from typing import Any

import pytest
from fastapi.testclient import TestClient

from tests.phrase_packages import lesson, make_package, phrase, publish
from tests.practice_client import Practice


def today(practice: Practice, lesson_id: str | None = None) -> dict[str, Any]:
    response = practice.client.get(
        "/api/practice/today", params={"lesson_id": lesson_id} if lesson_id else {}
    )
    assert response.status_code == 200
    return response.json()


def test_today_caps_counts_and_extra_extension_without_opening_a_view(client: TestClient) -> None:
    practice = Practice(client)
    publish(client, make_package(lessons=[lesson("one", "One", [phrase(i) for i in range(8)])]))
    ids = [row["id"] for row in practice.rows("SELECT id FROM phrases ORDER BY position")]
    practice.make_due(*ids[:4])
    practice.set_limits(due=2, new=2)
    assert today(practice) | {"message": ""} == {
        "learning_date": "2026-10-28",
        "due_count": 2,
        "new_count": 2,
        "extra_available": True,
        "message": "",
    }
    assert not practice.rows("SELECT * FROM current_view")
    state = practice.daily()
    while state["card"]:
        card = state["card"]
        state = practice.rate(card, "got_it" if card["kind"] == "recall" else "not_yet")["state"]
    assert today(practice)["due_count"] == today(practice)["new_count"] == 0
    assert today(practice)["extra_available"]
    practice.extra()
    assert today(practice)["due_count"] == today(practice)["new_count"] == 2
    assert today(practice)["extra_available"]


def test_today_skips_lesson_resume_phrase_practice_and_rollover(client: TestClient) -> None:
    practice = Practice(client)
    publish(
        client,
        make_package(
            lessons=[
                lesson("one", "One", [phrase(1), phrase(2)]),
                lesson("two", "Two", [phrase(3)]),
            ]
        ),
    )
    lessons = {row["title"]: row["id"] for row in client.get("/api/lessons").json()["items"]}
    assert today(practice, lessons["Two"])["new_count"] == 1
    card = practice.card(lessons["One"])
    practice.reveal(card)
    assert today(practice, lessons["Two"])["new_count"] == 2
    practice.set_limits(due=1, new=0)
    assert today(practice, lessons["Two"])["new_count"] == 1
    practice.set_limits(due=1, new=5)
    state = practice.finish(card, "skipped")["state"]
    assert today(practice, lessons["One"])["new_count"] == 1
    practice.finish(state["card"], "skipped")
    assert today(practice, lessons["One"])["new_count"] == 2
    practice.extra(lessons["One"])
    assert today(practice, lessons["One"])["new_count"] == 2
    practice.practise(card["phrase_id"])
    assert today(practice, lessons["Two"])["new_count"] == 1
    practice.later(days=1)
    assert today(practice, lessons["One"])["new_count"] == 2
    assert client.get("/api/practice/today?lesson_id=missing").status_code == 404


def test_today_zero_limits_and_no_extra_when_nothing_remains(client: TestClient) -> None:
    practice = Practice(client)
    assert today(practice)["message"] == "Add your first phrase with Coach to begin."
    publish(client, make_package(lessons=[lesson("one", "One", [phrase(1)])]))
    practice.set_limits(due=1, new=0)
    assert today(practice)["new_count"] == 0
    assert not today(practice)["extra_available"]
    practice.set_limits(due=0, new=1)
    practice.rate_all_new()
    assert today(practice)["new_count"] == today(practice)["due_count"] == 0
    assert not today(practice)["extra_available"]
    assert today(practice)["message"].startswith("Today's practice is complete.")


@pytest.mark.parametrize("kind", ["new", "recall"])
def test_extra_restores_skipped_phrases_before_the_allowance_is_used(
    client: TestClient, kind: str
) -> None:
    practice = Practice(client)
    publish(client, make_package(lessons=[lesson("one", "One", [phrase(1), phrase(2)])]))
    ids = [row["id"] for row in practice.rows("SELECT id FROM phrases ORDER BY position")]
    if kind == "recall":
        practice.make_due(*ids)
    practice.set_limits(due=5, new=5)
    state = practice.daily()
    while state["card"]:
        state = practice.finish(state["card"], "skipped")["state"]
    waiting = today(practice)
    assert (waiting["new_count"], waiting["due_count"]) == ((2, 0) if kind == "new" else (0, 2))
    assert waiting["extra_available"]
    before = practice.rows("SELECT * FROM phrases ORDER BY position")
    extra = practice.extra()["state"]
    assert extra["card"]["phrase_id"] in ids
    assert extra["card"]["kind"] == kind
    assert extra["progress"]["due_allowance"] == extra["progress"]["new_allowance"] == 5
    assert practice.log() == []
    after = practice.rows("SELECT * FROM phrases ORDER BY position")
    assert [row | {"last_cue": None} for row in after] == [
        row | {"last_cue": None} for row in before
    ]
