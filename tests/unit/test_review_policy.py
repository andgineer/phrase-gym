import json
from datetime import datetime, timedelta

from fsrs import State

from phrase_gym.practice.policy import ReviewPolicy
from tests.integration.helpers import NOW


def test_explicit_ratings_disabled_short_steps_and_real_elapsed_time():
    policy = ReviewPolicy()
    again, _ = policy.review(None, "not_yet", NOW)
    hard, hard_due = policy.review(None, "with_effort", NOW)
    good, good_due = policy.review(None, "recalled", NOW)
    assert all(json.loads(state)["state"] == State.Review for state in [again, hard, good])
    assert hard_due < good_due
    assert policy.scheduler.learning_steps == () and policy.scheduler.relearning_steps == ()
    state = good
    due = good_due
    for _ in range(15):
        state, due = policy.review(state, "recalled", due)
    assert due - datetime.fromisoformat(json.loads(state)["last_review"]) > timedelta(days=30)
    ontime, _ = policy.review(state, "recalled", due)
    overdue, _ = policy.review(state, "recalled", due + timedelta(days=90))
    assert json.loads(ontime)["stability"] != json.loads(overdue)["stability"]
    lapse, _ = policy.review(state, "not_yet", due)
    assert json.loads(lapse)["stability"] < json.loads(state)["stability"]
