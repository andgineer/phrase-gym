from collections.abc import Iterator
from contextlib import contextmanager
from datetime import UTC, datetime
from pathlib import Path
from typing import Any
from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

from phrase_gym.server.app import create_app
from tests.phrase_packages import Package, lesson, make_package, phrase, publish
from tests.practice_client import Practice

# Monday 10:00 in Europe/Belgrade; the week runs 2026-11-02 to 2026-11-08.
MONDAY = datetime(2026, 11, 2, 9, tzinfo=UTC)
OUTCOMES = ["got_it", "not_yet", "effort", "skipped", "easy"]


@pytest.fixture
def practice(client: TestClient) -> Practice:
    return Practice(client, now=MONDAY)


def many(count: int) -> Package:
    return make_package(lessons=[lesson("unit-1", "Meetings", [phrase(i) for i in range(count)])])


def facts(practice: Practice) -> dict[str, Any]:
    response = practice.client.get("/api/practice/facts")
    assert response.status_code == 200, response.text
    return response.json()


def week(practice: Practice) -> dict[str, Any]:
    return facts(practice)["week"]


def put_target(client: TestClient, target: int | None, paused: bool = False) -> Any:
    settings = client.get("/api/settings").json()
    settings |= {"weekly_target": target, "weekly_target_paused": paused}
    return client.put("/api/settings", json=settings)


def set_target(client: TestClient, target: int | None, paused: bool = False) -> None:
    response = put_target(client, target, paused)
    assert response.status_code == 200, response.text


def logged(practice: Practice, *days: str) -> None:
    with practice.database.write() as connection:
        connection.executemany(
            "INSERT INTO practice_log (id, phrase_id, phrase_text, lesson_title, completed_at, "
            "learning_date, kind, rating, skipped, fsrs_updated, mode) "
            "VALUES (?, NULL, 'Earlier phrase.', 'Earlier', ?, ?, 'recall', 'got_it', 0, 1, "
            "'daily')",
            [(str(uuid4()), f"{day}T09:00:00+00:00", day) for day in days],
        )


def complete(practice: Practice, card: dict[str, Any], outcome: str) -> dict[str, Any]:
    if outcome != "skipped":
        practice.reveal(card)
    return practice.finish(card, outcome)["state"]


def daily_completions(practice: Practice) -> int:
    state = practice.daily()
    count = 0
    while state["card"]:
        state = complete(practice, state["card"], OUTCOMES[count % len(OUTCOMES)])
        count += 1
    return count


def test_no_target_until_chosen_and_practice_is_fully_usable(practice: Practice) -> None:
    publish(practice.client, many(3))
    settings = practice.client.get("/api/settings").json()
    assert (settings["weekly_target"], settings["weekly_target_paused"]) == (None, False)
    assert week(practice) == {"practice_days": 0, "target": None}
    assert daily_completions(practice) == 3
    assert week(practice) == {"practice_days": 1, "target": None}
    assert facts(practice)["streak"]["current"] == 1


def test_target_is_chosen_changed_paused_and_resumed(practice: Practice) -> None:
    logged(practice, "2026-11-02")
    set_target(practice.client, 3)
    assert week(practice) == {"practice_days": 1, "target": 3}
    set_target(practice.client, 5)
    assert week(practice) == {"practice_days": 1, "target": 5}
    set_target(practice.client, 5, paused=True)
    assert week(practice) == {"practice_days": 1, "target": None}
    settings = practice.client.get("/api/settings").json()
    assert (settings["weekly_target"], settings["weekly_target_paused"]) == (5, True)
    set_target(practice.client, 5)
    assert week(practice) == {"practice_days": 1, "target": 5}
    set_target(practice.client, None)
    assert week(practice) == {"practice_days": 1, "target": None}


def test_a_pause_without_a_target_is_saved_and_read_as_not_paused(practice: Practice) -> None:
    response = put_target(practice.client, None, paused=True)
    assert response.status_code == 200, response.text
    assert response.json()["weekly_target_paused"] is False
    assert practice.client.get("/api/settings").json()["weekly_target_paused"] is False
    with practice.database.write() as connection:
        connection.execute(
            "UPDATE settings SET document=json_set(document, '$.weekly_target', json('null'), "
            "'$.weekly_target_paused', json('true'))"
        )
    stored = practice.client.get("/api/settings").json()
    assert (stored["weekly_target"], stored["weekly_target_paused"]) == (None, False)
    set_target(practice.client, 4)
    assert week(practice) == {"practice_days": 0, "target": 4}


def test_a_skip_a_historical_first_meeting_or_a_practice_rating_alone_counts_a_day(
    practice: Practice,
) -> None:
    publish(practice.client, many(3))
    set_target(practice.client, 3)
    card = practice.card()
    complete(practice, card, "skipped")
    practice.later(days=1)
    with practice.database.write() as connection:
        connection.execute(
            "INSERT INTO practice_log (id, phrase_id, phrase_text, lesson_title, completed_at, "
            "learning_date, kind, rating, skipped, fsrs_updated, mode) VALUES (?, NULL, "
            "'Earlier phrase.', 'Earlier', '2026-11-03T09:00:00+00:00', '2026-11-03', "
            "'first_study', NULL, 0, 0, 'daily')",
            (str(uuid4()),),
        )
    practice.later(days=1)
    complete(practice, practice.practise(card["phrase_id"]), "got_it")
    assert [
        (row["kind"], row["rating"], row["skipped"], row["mode"]) for row in practice.log()
    ] == [
        ("new", None, 1, "daily"),
        ("first_study", None, 0, "daily"),
        ("new", "got_it", 0, "practice"),
    ]
    found = facts(practice)
    assert found["week"] == {"practice_days": 3, "target": 3}
    assert found["streak"]["current"] == 3


@pytest.mark.parametrize("target", [0, 7, 2.5])
def test_target_outside_one_to_six_days_is_rejected(practice: Practice, target: object) -> None:
    set_target(practice.client, 2)
    settings = practice.client.get("/api/settings").json() | {"weekly_target": target}
    assert practice.client.put("/api/settings", json=settings).status_code == 422
    assert practice.client.get("/api/settings").json()["weekly_target"] == 2


def test_week_runs_monday_to_sunday_in_the_configured_timezone(practice: Practice) -> None:
    set_target(practice.client, 3)
    logged(practice, "2026-11-01", "2026-11-02", "2026-11-04", "2026-11-09")
    # Sunday 23:30 in Belgrade is still the week that began on Monday 2026-11-02.
    practice.now = datetime(2026, 11, 8, 22, 30, tzinfo=UTC)
    assert week(practice) == {"practice_days": 2, "target": 3}
    practice.later(hours=1)
    assert facts(practice)["learning_date"] == "2026-11-09"
    assert week(practice) == {"practice_days": 1, "target": 3}


def test_a_completion_keeps_its_week_when_the_timezone_changes(practice: Practice) -> None:
    publish(practice.client, many(4))
    set_target(practice.client, 2)
    practice.now = datetime(2026, 11, 8, 22, 30, tzinfo=UTC)
    complete(practice, practice.card(), "not_yet")
    assert practice.log()[0]["learning_date"] == "2026-11-08"
    assert week(practice) == {"practice_days": 1, "target": 2}
    settings = practice.client.get("/api/settings").json() | {"learning_timezone": "Asia/Tokyo"}
    assert practice.client.put("/api/settings", json=settings).status_code == 200
    # Already Monday in Tokyo: the Sunday completion stays in the previous week.
    assert facts(practice)["learning_date"] == "2026-11-09"
    assert week(practice) == {"practice_days": 0, "target": 2}
    assert facts(practice)["streak"]["current"] == 1
    complete(practice, practice.card(), "not_yet")
    assert practice.log()[-1]["learning_date"] == "2026-11-09"
    assert week(practice) == {"practice_days": 1, "target": 2}


def test_a_missed_day_leaves_even_the_largest_target_reachable(practice: Practice) -> None:
    publish(practice.client, many(30))
    set_target(practice.client, 6)
    practice.later(days=1)
    assert week(practice) == {"practice_days": 0, "target": 6}
    for day in range(6):
        assert daily_completions(practice) > 0
        assert week(practice) == {"practice_days": day + 1, "target": 6}
        practice.later(days=1)
    assert facts(practice)["learning_date"] == "2026-11-09"
    assert week(practice) == {"practice_days": 0, "target": 6}


def test_return_after_a_missed_week_carries_no_debt(practice: Practice) -> None:
    publish(practice.client, many(30))
    practice.set_limits(2, 2)
    set_target(practice.client, 3)
    for _ in range(3):
        assert daily_completions(practice) > 0
        practice.later(days=1)
    before = facts(practice)
    assert before["streak"]["best"] == 3
    # The whole week of 2026-11-09 passes without practice; the learner returns on Wednesday.
    practice.now = datetime(2026, 11, 18, 9, tzinfo=UTC)
    returned = facts(practice)
    assert returned["week"] == {"practice_days": 0, "target": 3}
    assert returned["streak"]["current"] == 0 and returned["streak"]["best"] == 3
    assert returned["work"] == before["work"]
    assert returned["library"]["phrase_count"] == before["library"]["phrase_count"]
    assert returned["library"]["due_count"] > 4
    state = practice.daily()
    assert state["progress"] == {
        "due_completed": 0,
        "due_allowance": 2,
        "new_completed": 0,
        "new_allowance": 2,
    }
    assert daily_completions(practice) == 4
    assert week(practice) == {"practice_days": 1, "target": 3}


@contextmanager
def fresh_app(directory: Path, monkeypatch: pytest.MonkeyPatch) -> Iterator[TestClient]:
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(directory))
    monkeypatch.setenv("PHRASE_GYM_FAKE_TTS", "1")
    with TestClient(create_app()) as client:
        yield client


def schedules(practice: Practice) -> dict[str, tuple[Any, ...]]:
    rows = practice.rows(
        "SELECT text, json_remove(fsrs_json, '$.card_id') AS fsrs, due_at, last_review, "
        "studied_at FROM phrases"
    )
    return {
        row["text"]: (row["fsrs"], row["due_at"], row["last_review"], row["studied_at"])
        for row in rows
    }


def history(practice: Practice) -> list[tuple[Any, ...]]:
    return [
        (row["phrase_text"], row["learning_date"], row["kind"], row["rating"], row["skipped"])
        + (row["fsrs_updated"], row["mode"])
        for row in practice.log()
    ]


def practice_trace(client: TestClient, adjust: list[tuple[int, bool]]) -> list[Any]:
    # Changing the target before every card shows a live change cannot leak into the next draw.
    practice = Practice(client, now=MONDAY)
    publish(client, many(12))
    practice.set_limits(3, 2)
    trace: list[Any] = []
    step = 0

    def run(state: dict[str, Any]) -> None:
        nonlocal step
        while state["card"]:
            if adjust:
                set_target(client, *adjust[step % len(adjust)])
            card = state["card"]
            revealed = practice.reveal(card)
            trace.append((card["mode"], card["kind"], card["cue"], revealed["answer"]["text"]))
            trace.append((state["progress"], revealed["return_days"]))
            state = practice.finish(card, OUTCOMES[step % len(OUTCOMES)])["state"]
            step += 1
        trace.append((state["progress"], state["message"]))

    for _ in range(3):
        run(practice.daily())
        run(practice.extra()["state"])
        practice.later(days=1)
    return [trace, schedules(practice), history(practice)]


def test_the_target_cannot_alter_grades_schedules_allowances_or_selection(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    variants: list[list[tuple[int, bool]]] = [[], [(3, False)], [(2, False), (6, False), (6, True)]]
    traces = []
    for index, adjust in enumerate(variants):
        with fresh_app(tmp_path / f"app-{index}", monkeypatch) as client:
            traces.append(practice_trace(client, adjust))
    assert len(traces[0][2]) > 15
    assert traces[1] == traces[0]
    assert traces[2] == traces[0]
