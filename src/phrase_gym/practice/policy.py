import json
from datetime import datetime
from importlib.metadata import version

from fsrs import Card, Rating, Scheduler

from phrase_gym.contracts.practice import RecallRating

POLICY_VERSION = f"fsrs-{version('fsrs')}-retention90-v1"
RATINGS = {"not_yet": Rating.Again, "with_effort": Rating.Hard, "recalled": Rating.Good}


class ReviewPolicy:
    def __init__(self):
        self.scheduler = Scheduler(
            desired_retention=0.90,
            learning_steps=(),
            relearning_steps=(),
            enable_fuzzing=False,
        )

    def review(
        self,
        state: str | None,
        rating: RecallRating,
        now: datetime,
    ) -> tuple[str, datetime]:
        card = Card.from_dict(json.loads(state)) if state else Card(card_id=0, due=now)
        updated, _log = self.scheduler.review_card(card, RATINGS[rating], review_datetime=now)
        return json.dumps(updated.to_dict(), sort_keys=True), updated.due

    def parameters(self) -> str:
        return json.dumps(self.scheduler.to_dict(), sort_keys=True)
