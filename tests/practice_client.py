import random
from datetime import UTC, datetime, timedelta
from typing import Any
from uuid import uuid4

import httpx
from fastapi.testclient import TestClient

from phrase_gym.server.storage import Database

Card = dict[str, Any]
CLOCKED = ("daily", "extra", "phrase_practice", "view_actions", "completion", "lessons")


class Practice:
    def __init__(self, client: TestClient, now: datetime | None = None, seed: int = 7):
        self.client = client
        self.now = now or datetime(2026, 10, 28, 10, tzinfo=UTC)
        self.rng = random.Random(seed)
        for name in CLOCKED:
            service = getattr(client.app.state, name)
            service.clock = lambda: self.now
            if hasattr(service, "rng"):
                service.rng = self.rng

    @property
    def database(self) -> Database:
        return self.client.app.state.database

    def later(self, **delta: float) -> None:
        self.now += timedelta(**delta)

    def daily(self, lesson_id: str | None = None) -> dict[str, Any]:
        response = self.client.post("/api/practice/daily", json={"lesson_id": lesson_id})
        assert response.status_code == 200, response.text
        return response.json()

    def card(self, lesson_id: str | None = None) -> Card:
        card = self.daily(lesson_id)["card"]
        assert card, "Daily offered no card"
        return card

    def press_extra(
        self, lesson_id: str | None = None, operation_id: str | None = None
    ) -> httpx.Response:
        body = {"operation_id": operation_id or str(uuid4()), "lesson_id": lesson_id}
        return self.client.post("/api/practice/extra", json=body)

    def extra(
        self, lesson_id: str | None = None, operation_id: str | None = None
    ) -> dict[str, Any]:
        response = self.press_extra(lesson_id, operation_id)
        assert response.status_code == 200, response.text
        return response.json()

    def practise(self, phrase_id: str) -> Card:
        response = self.client.post("/api/practice/phrase", json={"phrase_id": phrase_id})
        assert response.status_code == 200, response.text
        card = response.json()["card"]
        assert card and card["mode"] == "practice"
        return card

    def action(self, path: str, card: Card, **extra: object) -> httpx.Response:
        body = {"view_token": card["view_token"], "phrase_id": card["phrase_id"], **extra}
        return self.client.post(f"/api/practice/{path}", json=body)

    def reveal(self, card: Card) -> Card:
        response = self.action("reveal", card)
        assert response.status_code == 200, response.text
        return response.json()

    def choose(self, path: str, card: Card, operation_id: str | None = None) -> Card:
        response = self.action(path, card, operation_id=operation_id or str(uuid4()))
        assert response.status_code == 200, response.text
        return response.json()

    def complete(
        self, card: Card, outcome: str, completion_id: str | None = None
    ) -> httpx.Response:
        return self.action(
            "complete", card, outcome=outcome, completion_id=completion_id or str(uuid4())
        )

    def finish(self, card: Card, outcome: str, completion_id: str | None = None) -> dict[str, Any]:
        response = self.complete(card, outcome, completion_id)
        assert response.status_code == 200, response.text
        return response.json()

    def study_all_new(self) -> None:
        state = self.daily()
        while state["card"] and state["card"]["kind"] == "first_study":
            state = self.finish(state["card"], "studied")["state"]

    def rows(self, sql: str, *parameters: object) -> list[dict[str, Any]]:
        with self.database.read() as connection:
            return [dict(row) for row in connection.execute(sql, parameters).fetchall()]

    def log(self) -> list[dict[str, Any]]:
        return self.rows("SELECT * FROM practice_log ORDER BY completed_at, rowid")

    def phrase(self, phrase_id: str) -> dict[str, Any]:
        return self.rows("SELECT * FROM phrases WHERE id=?", phrase_id)[0]

    def make_due(self, *phrase_ids: str, since: timedelta = timedelta(days=2)) -> None:
        earlier = (self.now - since).isoformat()
        with self.database.write() as connection:
            for phrase_id in phrase_ids:
                connection.execute(
                    "UPDATE phrases SET studied_at=?, due_at=? WHERE id=?",
                    (earlier, earlier, phrase_id),
                )

    def set_limits(self, due: int, new: int) -> None:
        settings = self.client.get("/api/settings").json()
        settings |= {"daily_due_limit": due, "daily_new_limit": new}
        assert self.client.put("/api/settings", json=settings).status_code == 200
