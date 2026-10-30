from typing import Any

import pytest
from fastapi.testclient import TestClient

from tests.phrase_packages import lesson, make_package, phrase, publish
from tests.practice_client import Practice


@pytest.fixture
def practice(client: TestClient) -> Practice:
    publish(
        client,
        make_package(lessons=[lesson("unit-1", "Meetings", [phrase(i) for i in range(1, 4)])]),
    )
    return Practice(client)


def current(practice: Practice) -> dict[str, Any]:
    response = practice.client.get("/api/practice/current")
    assert response.status_code == 200, response.text
    return response.json()


def stored(practice: Practice) -> list[list[dict[str, Any]]]:
    return [
        practice.rows(f"SELECT * FROM {table} ORDER BY rowid")
        for table in ("current_view", "phrases", "practice_log", "extra_targets", "operations")
    ]


def test_current_returns_todays_regular_view_without_changing_anything(
    practice: Practice,
) -> None:
    assert current(practice)["card"] is None
    assert stored(practice)[0] == []
    card = practice.extra()["state"]["card"]
    before = stored(practice)
    state = current(practice)
    assert state["card"] == card and state["message"] is None
    assert state["progress"] == practice.daily()["progress"]
    assert stored(practice) == before


def test_current_leaves_an_open_practice_view_and_its_traversal_alone(practice: Practice) -> None:
    lessons = practice.client.get("/api/lessons").json()["items"]
    daily = practice.card(lessons[0]["id"])
    card = practice.practise(practice.rows("SELECT id FROM phrases ORDER BY position")[2]["id"])
    before = stored(practice)
    assert current(practice)["card"] is None
    assert stored(practice) == before
    assert practice.finish(card, "studied")["receipt"]["mode"] == "practice"
    assert practice.complete(daily, "studied").status_code == 409


def test_current_does_not_close_or_replace_an_old_date_view(practice: Practice) -> None:
    old = practice.card()
    practice.later(days=1)
    before = stored(practice)
    state = current(practice)
    assert state["card"] is None and state["progress"]["new_completed"] == 0
    assert stored(practice) == before
    assert before[0][0]["view_token"] == old["view_token"]


def test_current_treats_a_view_whose_phrase_is_gone_as_no_card(practice: Practice) -> None:
    card = practice.card()
    with practice.database.write() as connection:
        connection.execute("UPDATE current_view SET phrase_id='gone-phrase-1'")
    state = current(practice)
    assert state["card"] is None and state["progress"]["new_completed"] == 0
    assert practice.rows("SELECT view_token FROM current_view") == [
        {"view_token": card["view_token"]}
    ]
