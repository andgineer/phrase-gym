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


def silent_reference() -> str:
    return base64.b64encode(asyncio.run(PiperSpeech(None, fake=True).synthesize("x"))).decode()


def coach_card(item_id: str, **changes: object) -> dict[str, object]:
    return {
        "item_id": item_id,
        "text": "We strongly believe this will help.",
        "explanation": "Target chunk: strongly believe. Say you are sure about an opinion.",
        "cues": [
            {"situation": "We __________ that the plan will work. (=We truly think)"},
            {"situation": "I __________ she is right about this. (=I am sure)"},
        ],
        "recall_hints": ["strong / bel...", "strongly / be..."],
        "usage_contexts": ["opinions"],
        "accepted_variants": ["firmly believe"],
        **changes,
    }


def prepare_typed(client: TestClient, typed: str, **changes: object) -> dict[str, Any]:
    submitted = client.post("/api/coach/requests", json={"text": typed}).json()
    work = client.post("/api/imports/coach/next").json()
    assert work == {"package_id": submitted["request_id"], "text": typed.strip()}
    response = client.post(
        f"/api/imports/{work['package_id']}/result",
        json={
            "result_id": work["package_id"],
            "phrase": coach_card(work["package_id"], **changes),
            "reference": silent_reference(),
        },
    )
    assert response.status_code == 200, response.text
    return response.json()


def upload_take(
    client: TestClient,
    card: dict[str, Any],
    take_id: str,
    audio: bytes = b"complete saved take",
    captured_at: str = "2026-10-28T10:00:00+00:00",
    **snapshot: object,
) -> Any:
    return client.post(
        "/api/recordings",
        json={
            "id": take_id,
            "phrase_id": card["phrase_id"],
            "view_token": card["view_token"],
            "mode": card["mode"],
            "kind": card["kind"],
            "cue": card["cue"],
            "hint": card["hint"],
            "captured_at": captured_at,
            "mime_type": "audio/webm",
            "duration_ms": 700,
            "complete": True,
            "audio_base64": base64.b64encode(audio).decode(),
            **snapshot,
        },
    )
