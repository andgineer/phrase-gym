import json

from tests.preservation import fsrs_cases
from tests.preservation.export import FSRS_FIXTURE


def test_committed_scheduler_fixtures_match_the_locked_implementation():
    committed = json.loads(FSRS_FIXTURE.read_text(encoding="utf-8"))
    assert committed == fsrs_cases.fixture()


def test_fixtures_cover_required_scheduler_situations():
    document = fsrs_cases.fixture()
    scheduler = document["scheduler"]
    assert scheduler["desired_retention"] == 0.9
    assert scheduler["learning_steps"] == [] and scheduler["relearning_steps"] == []
    assert scheduler["enable_fuzzing"] is False
    states = [
        json.loads(review["state"]) for case in document["cases"] for review in case["reviews"]
    ]
    assert {review["rating"] for case in document["cases"] for review in case["reviews"]} == {
        "not_yet",
        "with_effort",
        "recalled",
    }
    assert any(state["stability"] == 0.001 for state in states)
    assert any(state["difficulty"] == 10.0 or state["difficulty"] > 9.9 for state in states)
    maximum = next(case for case in document["cases"] if case["name"] == "maximum-interval")
    last = json.loads(maximum["reviews"][-2]["state"])
    assert last["due"] > last["last_review"]
