"""Synthetic review sequences recorded from the locked scheduler for the Rust replacement."""

import json
from collections.abc import Callable
from datetime import UTC, datetime, timedelta
from importlib.metadata import version

from phrase_gym.practice.policy import POLICY_VERSION, RATINGS, ReviewPolicy

START = datetime(2026, 10, 12, 12, tzinfo=UTC)
Moment = Callable[[datetime | None, datetime | None], datetime]


def at_due(offset: timedelta = timedelta()) -> Moment:
    return lambda due, _last: (due or START) + offset


def after_last(offset: timedelta) -> Moment:
    return lambda _due, last: (last or START) + offset


def fixed(moment: datetime) -> Moment:
    return lambda _due, _last: moment


def early(fraction: float) -> Moment:
    return lambda due, last: last + (due - last) * fraction


def record(policy: ReviewPolicy, name: str, plan: list[tuple[str, Moment]]) -> dict:
    state, due, last = None, None, None
    reviews = []
    for rating, moment in plan:
        at = moment(due, last)
        state, due = policy.review(state, rating, at)
        last = at
        reviews.append(
            {"rating": rating, "at": at.isoformat(), "state": state, "due": due.isoformat()},
        )
    return {"name": name, "reviews": reviews}


def until_maximum_interval(policy: ReviewPolicy) -> list[tuple[str, Moment]]:
    plan: list[tuple[str, Moment]] = [("recalled", fixed(START))]
    state, due = policy.review(None, "recalled", START)
    last = START
    while due - last < timedelta(days=policy.scheduler.maximum_interval):
        last = due
        state, due = policy.review(state, "recalled", due)
        plan.append(("recalled", at_due()))
    return plan + [("recalled", at_due()), ("with_effort", at_due())]


def cases(policy: ReviewPolicy) -> list[dict]:
    minute = timedelta(minutes=1)
    plans: dict[str, list[tuple[str, Moment]]] = {
        "first-not-yet": [("not_yet", fixed(START))],
        "first-with-effort": [("with_effort", fixed(START))],
        "first-recalled": [("recalled", fixed(START))],
        "recalled-at-due": [("recalled", at_due())] * 16,
        "with-effort-at-due": [("with_effort", at_due())] * 10,
        "mixed-at-due": [
            (rating, at_due())
            for rating in ["recalled", "with_effort", "recalled", "not_yet", "recalled"]
            + ["recalled", "with_effort", "not_yet", "with_effort", "recalled"]
        ],
        "lapse-after-long-run": [("recalled", at_due())] * 8
        + [("not_yet", at_due())]
        + [("recalled", at_due())] * 4,
        "repeated-lapses": [("not_yet", at_due())] * 12,
        "long-gaps": [
            ("recalled", fixed(START)),
            ("recalled", at_due(timedelta(days=90))),
            ("recalled", at_due(timedelta(days=400))),
            ("not_yet", at_due(timedelta(days=1095))),
            ("recalled", at_due(timedelta(days=30))),
            ("with_effort", at_due(timedelta(days=2000))),
        ],
        "same-day-reviews": [
            ("recalled", fixed(START)),
            ("recalled", after_last(10 * minute)),
            ("with_effort", after_last(10 * minute)),
            ("not_yet", after_last(10 * minute)),
            ("recalled", after_last(timedelta(days=1) - timedelta(microseconds=1))),
            ("recalled", at_due()),
            ("not_yet", after_last(5 * minute)),
        ],
        "day-boundary": [
            ("recalled", fixed(START)),
            ("recalled", after_last(timedelta(days=1))),
            ("recalled", after_last(timedelta(days=1) - timedelta(microseconds=1))),
            ("with_effort", after_last(timedelta(days=2))),
        ],
        "early-reviews": [("recalled", at_due())] * 5
        + [("recalled", early(0.5)), ("with_effort", early(0.25)), ("recalled", at_due())],
        "microsecond-times": [
            ("recalled", fixed(START + timedelta(microseconds=123456))),
            ("with_effort", at_due(timedelta(hours=7, microseconds=654321))),
            ("not_yet", at_due(timedelta(microseconds=1))),
            ("recalled", after_last(timedelta(minutes=3, microseconds=999999))),
        ],
        "same-day-failures": [("recalled", fixed(START))]
        + [("not_yet", after_last(minute))] * 60
        + [("recalled", after_last(timedelta(days=2)))],
        "maximum-interval": until_maximum_interval(policy),
    }
    return [record(policy, name, plan) for name, plan in plans.items()]


def fixture() -> dict:
    policy = ReviewPolicy()
    return {
        "source": f"phrase_gym.practice.policy.ReviewPolicy with fsrs {version('fsrs')}",
        "policy_version": POLICY_VERSION,
        "scheduler": json.loads(policy.parameters()),
        "ratings": {name: int(rating) for name, rating in RATINGS.items()},
        "cases": cases(policy),
    }
