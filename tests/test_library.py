import sqlite3
import time
from datetime import UTC, datetime, timedelta
from pathlib import Path
from uuid import uuid4

from fastapi.testclient import TestClient

from phrase_gym.server.storage import Database
from phrase_gym.server.study import StudyService
from tests.phrase_packages import lesson, make_package, phrase, publish, upload_take

TABLES = ("phrases", "lessons", "attempts", "reviews", "operations", "recordings")


def snapshot(path: Path) -> dict[str, list[tuple[object, ...]]]:
    with sqlite3.connect(path / "phrase-gym.sqlite3") as connection:
        return {
            table: connection.execute(f"SELECT * FROM {table} ORDER BY rowid").fetchall()
            for table in TABLES
        }


def varied(index: int) -> dict[str, object]:
    return phrase(
        index,
        text=f"Useful expression number {index}.",
        usage_contexts=["small talk"] if index % 7 == 0 else ["meetings"],
        explanation=(
            "Use it to talk about a deadline."
            if index % 11 == 0
            else "Use it naturally in conversation."
        ),
        cues=[
            {"situation": "A colleague mentions the elevator again."}
            if index % 13 == 0
            else {"situation": f"Situation number {index}."}
        ],
        detailed_explanation="«Пока всё хорошо»." if index % 17 == 0 else None,
        recall_hints=["zebra", f"clue {index}"],
    )


def thousand_phrases(client: TestClient) -> None:
    for block in range(20):
        start = block * 50
        publish(
            client,
            make_package(
                f"bulk-{block}",
                [
                    lesson(f"a-{block}", f"Unit {block} A", [varied(start + i) for i in range(25)]),
                    lesson(
                        f"b-{block}", f"Unit {block} B", [varied(start + i) for i in range(25, 50)]
                    ),
                ],
                f"bulk-source-{block}",
            ),
        )


def search(client: TestClient, query: str, **params: object) -> list[dict[str, object]]:
    found: list[dict[str, object]] = []
    offset: int | None = 0
    while offset is not None:
        page = client.get(
            "/api/phrases", params={"query": query, "offset": offset, "limit": 100, **params}
        ).json()
        found.extend(page["items"])
        offset = page["next_offset"]
    return found


def test_thousand_phrase_mixed_lesson_search(client: TestClient) -> None:
    thousand_phrases(client)
    started = time.perf_counter()
    small_talk = search(client, "Small TALK")
    assert len(small_talk) == len([i for i in range(1000) if i % 7 == 0])
    assert len({item["lesson_title"] for item in small_talk}) == 40
    assert all(item["usage_contexts"] == ["small talk"] for item in small_talk)

    talk = search(client, "talk")
    primary = [i for i in range(1000) if i % 7 == 0]
    explanation_only = [i for i in range(1000) if i % 11 == 0 and i % 7 != 0]
    assert len(talk) == len(primary) + len(explanation_only)
    assert all(item["usage_contexts"] == ["small talk"] for item in talk[: len(primary)])
    assert all(item["usage_contexts"] == ["meetings"] for item in talk[len(primary) :])

    assert len(search(client, "elevator")) == len([i for i in range(1000) if i % 13 == 0])
    assert len(search(client, "dead")) == len([i for i in range(1000) if i % 11 == 0])
    assert search(client, "eadline") == []
    assert len(search(client, "пока")) == len([i for i in range(1000) if i % 17 == 0])
    assert search(client, "zebra") == []
    exact = search(client, "expression number 994")
    assert [item["text"] for item in exact] == ["Useful expression number 994."]
    assert time.perf_counter() - started < 5

    first_page = client.get("/api/phrases", params={"limit": 20}).json()
    assert first_page["next_offset"] == 20
    assert first_page["items"][0]["text"] == "Useful expression number 0."
    lessons = client.get("/api/lessons", params={"query": "unit 3 b"}).json()
    assert [item["title"] for item in lessons["items"]] == ["Unit 3 B"]
    only = search(client, "", lesson_id=lessons["items"][0]["id"])
    assert len(only) == 25
    assert {item["lesson_title"] for item in only} == {"Unit 3 B"}


def test_lesson_choices_are_paged_searchable_and_counted(
    client: TestClient, tmp_path: Path
) -> None:
    publish(
        client,
        make_package(
            lessons=[
                lesson("greetings", "Greetings", [phrase(1), phrase(2)]),
                lesson("small-talk", "Small talk", [phrase(3)]),
                lesson("meetings", "Meetings", [phrase(4)]),
            ]
        ),
    )
    instant = datetime(2026, 10, 28, 10, tzinfo=UTC)
    client.app.state.lessons.clock = lambda: instant
    study = StudyService(Database(tmp_path), lambda: instant)
    card = study.current().card
    assert card and card.text == "Complete phrase number 1."
    study.complete(card.attempt_id, str(uuid4()))

    first = client.get("/api/lessons", params={"limit": 2}).json()
    assert [item["title"] for item in first["items"]] == ["Greetings", "Small talk"]
    assert first["next_offset"] == 2
    assert first["items"][0] | {"id": ""} == {
        "id": "",
        "title": "Greetings",
        "source_label": "Course · Meetings",
        "phrase_count": 2,
        "due_count": 0,
        "new_count": 1,
    }
    assert first["all_lessons"] == {"phrase_count": 4, "due_count": 0, "new_count": 3}
    rest = client.get("/api/lessons", params={"offset": 2, "limit": 2}).json()
    assert [item["title"] for item in rest["items"]] == ["Meetings"]
    assert rest["next_offset"] is None
    assert client.get("/api/lessons", params={"query": "sma"}).json()["items"][0]["title"] == (
        "Small talk"
    )
    assert client.get("/api/lessons", params={"query": "nothing"}).json()["items"] == []

    instant += timedelta(days=1)
    tomorrow = client.get("/api/lessons").json()
    assert tomorrow["items"][0]["due_count"] == 1
    assert tomorrow["all_lessons"]["due_count"] == 1
    assert client.get("/api/lessons", params={"limit": 0}).status_code == 422
    assert client.get("/api/lessons", params={"limit": 101}).status_code == 422


def test_details_show_lesson_source_reference_due_and_paged_recordings(
    client: TestClient, tmp_path: Path
) -> None:
    details_phrase = phrase(
        1,
        text="So far, so good.",
        usage_contexts=["small talk", "progress update"],
        explanation="Things have gone well up to now.",
        detailed_explanation="«Пока всё хорошо».",
        accepted_variants=["So far so good."],
    )
    publish(client, make_package(lessons=[lesson("small-talk", "Small talk", [details_phrase])]))
    instant = datetime(2026, 10, 28, 10, tzinfo=UTC)
    study = StudyService(Database(tmp_path), lambda: instant)
    card = study.current().card
    assert card
    before = client.get(f"/api/phrases/{card.id}").json()
    assert before["studied"] is False and before["due_date"] is None
    take_ids = [str(uuid4()) for _ in range(3)]
    for take_id in take_ids:
        assert upload_take(client, card.model_dump(), take_id).status_code == 200
    study.complete(card.attempt_id, str(uuid4()))

    details = client.get(f"/api/phrases/{card.id}").json()
    assert details | {"id": "", "lesson_id": ""} == {
        "id": "",
        "lesson_id": "",
        "text": "So far, so good.",
        "explanation": "Things have gone well up to now.",
        "detailed_explanation": "«Пока всё хорошо».",
        "usage_contexts": ["small talk", "progress update"],
        "accepted_variants": ["So far so good."],
        "lesson_title": "Small talk",
        "source_label": "Course · Meetings",
        "source_link": "https://example.test/meetings",
        "reference_url": f"/api/phrases/{card.id}/reference",
        "studied": True,
        "due_date": "2026-10-29",
    }
    assert client.get(details["reference_url"]).content.startswith(b"RIFF")
    page = client.get(f"/api/phrases/{card.id}/recordings", params={"limit": 2}).json()
    assert len(page["items"]) == 2 and page["next_offset"] == 2
    assert page["items"][0]["mode"] == "study"
    assert page["items"][0]["answer"] == "So far, so good."
    more = client.get(f"/api/phrases/{card.id}/recordings", params={"offset": 2}).json()
    assert len(more["items"]) == 1 and more["next_offset"] is None
    assert {item["id"] for item in page["items"] + more["items"]} == set(take_ids)
    assert client.get("/api/phrases/missing-phrase").status_code == 404
    assert client.get("/api/phrases/missing-phrase/recordings").status_code == 404


def test_lookup_makes_no_scheduling_writes(client: TestClient, tmp_path: Path) -> None:
    publish(client, make_package())
    instant = datetime(2026, 10, 28, 10, tzinfo=UTC)
    study = StudyService(Database(tmp_path), lambda: instant)
    first = study.current().card
    assert first
    study.complete(first.attempt_id, str(uuid4()))
    instant += timedelta(days=1)
    review = study.current().card
    assert review and review.kind == "review"
    before = snapshot(tmp_path)
    client.get("/api/lessons", params={"query": "meet"})
    client.get("/api/phrases", params={"query": "phrase number"})
    client.get(f"/api/phrases/{review.id}")
    client.get(f"/api/phrases/{review.id}/recordings")
    client.get(f"/api/phrases/{review.id}/reference")
    assert snapshot(tmp_path) == before
    assert study.current().card.attempt_id == review.attempt_id
