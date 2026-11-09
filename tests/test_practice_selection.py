from datetime import timedelta
from uuid import uuid4

from fastapi.testclient import TestClient

from phrase_gym.server.app import create_app
from tests.phrase_packages import lesson, make_package, phrase, publish
from tests.practice_client import Practice


def publish_many(client: TestClient, count: int) -> list[str]:
    publish(
        client,
        make_package(lessons=[lesson("unit-1", "Meetings", [phrase(i) for i in range(count)])]),
    )
    with client.app.state.database.read() as connection:
        return [row[0] for row in connection.execute("SELECT id FROM phrases ORDER BY position")]


def first_cards(client: TestClient, presses: int) -> list[tuple[str, str]]:
    cards = []
    for _ in range(presses):
        body = {"operation_id": str(uuid4()), "lesson_id": None}
        response = client.post("/api/practice/extra", json=body)
        assert response.status_code == 200, response.text
        card = response.json()["state"]["card"]
        cards.append((card["kind"], card["phrase_id"]))
    return cards


def test_fresh_sessions_can_start_with_any_eligible_phrase(client: TestClient) -> None:
    ids = publish_many(client, 20)
    Practice(client).make_due(*ids[:10])
    drawn = first_cards(client, 300)
    assert {item for kind, item in drawn if kind == "recall"} == set(ids[:10])
    assert {item for kind, item in drawn if kind == "new"} == set(ids[10:])
    due_share = sum(kind == "recall" for kind, _ in drawn) / len(drawn)
    # Ten due slots against five new slots put a review first in two sessions of three.
    assert 0.58 < due_share < 0.75


def test_longest_due_phrases_fill_the_due_allowance_in_random_order(client: TestClient) -> None:
    ids = publish_many(client, 6)
    practice = Practice(client)
    practice.make_due(*ids[3:], since=timedelta(days=5))
    practice.make_due(*ids[:3], since=timedelta(days=1))
    practice.set_limits(due=3, new=1)
    drawn = first_cards(client, 100)
    assert {item for kind, item in drawn if kind == "recall"} == set(ids[3:])


def test_separately_started_servers_draw_different_orders(client: TestClient) -> None:
    publish_many(client, 20)
    first = first_cards(client, 30)
    with TestClient(create_app()) as restarted:
        second = first_cards(restarted, 30)
    assert len(set(first)) > 1 and len(set(second)) > 1
    assert first != second
