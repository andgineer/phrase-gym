from datetime import datetime
from zoneinfo import ZoneInfo

import pytest
from fastapi.testclient import TestClient

from tests.phrase_packages import Package, publish
from tests.practice_client import Card, Practice

ZONE = ZoneInfo("Europe/Belgrade")


@pytest.fixture
def practice(client: TestClient, package: Package) -> Practice:
    publish(client, package)
    return Practice(client)


def days_until_due(practice: Practice, phrase_id: str) -> int:
    due = datetime.fromisoformat(practice.phrase(phrase_id)["due_at"]).astimezone(ZONE)
    return (due.date() - practice.now.astimezone(ZONE).date()).days


def set_pace(practice: Practice, pace: str) -> None:
    settings = practice.client.get("/api/settings").json() | {"return_pace": pace}
    assert practice.client.put("/api/settings", json=settings).status_code == 200


def revealed_new_card(practice: Practice) -> Card:
    card = practice.card()
    assert card["kind"] == "new" and card["return_days"] is None
    return practice.reveal(card)


def test_return_pace_defaults_to_balanced_and_counts_good_returns(practice: Practice) -> None:
    assert practice.client.get("/api/settings").json()["return_pace"] == "balanced"
    assert practice.client.get("/api/settings/return-paces").json() == [
        {"pace": "often", "first_month_returns": 5, "first_year_returns": 9},
        {"pace": "balanced", "first_month_returns": 3, "first_year_returns": 7},
        {"pace": "rarely", "first_month_returns": 2, "first_year_returns": 4},
    ]
    settings = practice.client.get("/api/settings").json() | {"return_pace": "never"}
    assert practice.client.put("/api/settings", json=settings).status_code == 422


@pytest.mark.parametrize(
    "pace, expected",
    [
        ("balanced", {"not_yet": 1, "effort": 1, "got_it": 1, "easy": 3}),
        ("rarely", {"not_yet": 1, "effort": 1, "got_it": 2, "easy": 8}),
    ],
)
def test_new_card_forecast_follows_the_chosen_pace(
    practice: Practice, pace: str, expected: dict[str, int]
) -> None:
    set_pace(practice, pace)
    card = revealed_new_card(practice)
    assert card["return_days"] == expected
    practice.finish(card, "easy")
    assert days_until_due(practice, card["phrase_id"]) == expected["easy"]


@pytest.mark.parametrize("rating", ["not_yet", "effort", "got_it", "easy"])
def test_recall_forecast_matches_the_return_the_grade_sets(practice: Practice, rating: str) -> None:
    practice.rate_all_new()
    practice.later(days=3)
    card = practice.card()
    assert card["kind"] == "recall" and card["return_days"] is None
    forecast = practice.reveal(card)["return_days"]
    practice.finish(card, rating)
    assert days_until_due(practice, card["phrase_id"]) == forecast[rating]


def test_a_phrase_graded_today_shows_no_forecast(practice: Practice) -> None:
    card = revealed_new_card(practice)
    practice.finish(card, "got_it")
    again = practice.practise(card["phrase_id"])
    assert practice.reveal(again)["return_days"] is None
    practice.later(days=1)
    tomorrow = practice.practise(card["phrase_id"])
    assert practice.reveal(tomorrow)["return_days"] is not None
