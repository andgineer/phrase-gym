import asyncio
import base64
from typing import Any

from fastapi.testclient import TestClient

from phrase_gym.coach.speech import PiperSpeech

Package = dict[str, Any]


def phrase(index: int, **changes: object) -> dict[str, object]:
    return {
        "item_id": f"meeting-{index}",
        "text": f"Complete phrase number {index}.",
        "explanation": "Use it to communicate naturally in a meeting.",
        "cues": [{"situation": f"Meeting situation number {index}."}],
        "recall_hints": [f"hint {index}", f"clue {index}"],
        "usage_contexts": ["meetings"],
        "accepted_variants": [],
        **changes,
    }


def lesson(lesson_id: str, title: str, phrases: list[dict[str, object]]) -> dict[str, object]:
    return {"lesson_id": lesson_id, "title": title, "phrases": phrases}


def make_package(
    package_id: str = "course-meetings-v1",
    lessons: list[dict[str, object]] | None = None,
    source_id: str = "course-meetings",
) -> Package:
    return {
        "format": "phrase-gym.package",
        "version": 2,
        "package_id": package_id,
        "source_id": source_id,
        "source_label": "Course · Meetings",
        "source_link": "https://example.test/meetings",
        "learning_language": "en",
        "card_language": "en",
        "help_language": "en",
        "lessons": lessons
        or [lesson("unit-1", "Meetings", [phrase(index) for index in range(1, 6)])],
    }


def phrases_of(package: Package) -> list[dict[str, Any]]:
    return [item for unit in package["lessons"] for item in unit["phrases"]]


def references(package: Package) -> dict[str, str]:
    speech = PiperSpeech(None, fake=True)
    return {
        item["item_id"]: base64.b64encode(asyncio.run(speech.synthesize(item["text"]))).decode()
        for item in phrases_of(package)
    }


def publish(client: TestClient, package: Package) -> dict[str, Any]:
    response = client.post(
        "/api/imports",
        json={"package": package, "references": references(package)},
    )
    assert response.status_code == 202, response.text
    assert response.json()["status"] == "added"
    return response.json()


def prepare_with_coach(client: TestClient, package: Package) -> dict[str, Any]:
    assert client.post("/api/imports", json={"package": package}).status_code == 202
    work = client.post("/api/imports/coach/next").json()
    assert work["package_id"] == package["package_id"]
    response = client.post(
        f"/api/imports/{package['package_id']}/result",
        json={"result_id": package["package_id"], "references": references(package)},
    )
    assert response.status_code == 200, response.text
    return response.json()


def upload_take(
    client: TestClient,
    card: dict[str, Any],
    take_id: str,
    audio: bytes = b"complete saved take",
) -> Any:
    return client.post(
        "/api/recordings",
        json={
            "id": take_id,
            "phrase_id": card["id"],
            "attempt_id": card["attempt_id"],
            "captured_at": "2026-10-28T10:00:00+00:00",
            "mime_type": "audio/webm",
            "duration_ms": 700,
            "complete": True,
            "audio_base64": base64.b64encode(audio).decode(),
        },
    )
