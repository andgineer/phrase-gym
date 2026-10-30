import base64
import io
import json
import urllib.error
import wave
from email.message import Message
from pathlib import Path
from typing import Any

import pytest
from fastapi.testclient import TestClient

import phrase_gym.server.publication as publication
from phrase_gym.coach.main import process_once
from phrase_gym.coach.speech import PiperSpeech
from phrase_gym.server.app import create_app
from phrase_gym.server.import_contracts import PhrasePackage
from phrase_gym.server.importer import Importer
from phrase_gym.server.storage import Database
from tests.phrase_packages import (
    Package,
    lesson,
    make_package,
    phrase,
    phrases_of,
    prepare_with_coach,
    publish,
    references,
)


def table_count(database: Database, table: str) -> int:
    with database.read() as connection:
        return connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]


def test_offline_restart_retry_and_idempotent_publication(
    tmp_path: Path, package: Package, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    with TestClient(create_app()) as client:
        accepted = client.post("/api/imports", json={"package": package})
        assert accepted.status_code == 202
        assert accepted.json()["status"] == "pending"
        assert client.get("/api/lessons").json()["all_lessons"]["phrase_count"] == 0
        assert client.post("/api/imports", json={"package": package}).json() == accepted.json()
        work = client.post("/api/imports/coach/next").json()
        assert work["package_id"] == package["package_id"]
    with TestClient(create_app()) as client:
        assert client.get(f"/api/imports/{package['package_id']}").json()["status"] == "pending"
        assert client.post("/api/imports/coach/next").json()["package_id"] == package["package_id"]
        result = {"result_id": package["package_id"], "references": references(package)}
        first = client.post(f"/api/imports/{package['package_id']}/result", json=result)
        assert first.json()["added_count"] == 5
        assert (
            client.post(f"/api/imports/{package['package_id']}/result", json=result).json()
            == first.json()
        )
        assert client.post("/api/imports/coach/next").json() is None
        assert client.post("/api/practice/daily", json={}).json()["card"]["answer"]["text"]
    assert table_count(Database(tmp_path), "phrases") == 5
    assert len(list((tmp_path / "references").iterdir())) == 5


def test_interrupted_preparation_can_be_retried_without_server_restart(
    client: TestClient, package: Package
) -> None:
    package_id = package["package_id"]
    client.post("/api/imports", json={"package": package})
    assert client.post("/api/imports/coach/next").json()["package_id"] == package_id
    assert client.post(f"/api/imports/{package_id}/retry").json()["status"] == "pending"
    assert client.post("/api/imports/coach/next").json()["package_id"] == package_id
    result = {"result_id": package_id, "references": references(package)}
    assert client.post(f"/api/imports/{package_id}/result", json=result).json()["added_count"] == 5
    assert client.post(f"/api/imports/{package_id}/result", json=result).json()["added_count"] == 5


def test_malformed_audio_and_missing_item_reject_whole_batch(
    client: TestClient, package: Package
) -> None:
    package_id = package["package_id"]
    client.post("/api/imports", json={"package": package})
    client.post("/api/imports/coach/next")
    valid = references(package)
    missing = valid.copy()
    missing.pop("meeting-3")
    response = client.post(
        f"/api/imports/{package_id}/result", json={"result_id": package_id, "references": missing}
    )
    assert response.status_code == 409
    assert "meeting-3" in response.text
    malformed = valid.copy()
    malformed["meeting-3"] = base64.b64encode(b"not wave").decode()
    response = client.post(
        f"/api/imports/{package_id}/result", json={"result_id": package_id, "references": malformed}
    )
    assert response.status_code == 409
    assert "phrases.meeting-3" in response.text
    assert table_count(client.app.state.database, "phrases") == 0
    assert table_count(client.app.state.database, "lessons") == 0
    failed = client.post(f"/api/imports/{package_id}/failure", json={"error": "Piper unavailable"})
    assert failed.json()["status"] == "error"
    assert client.get("/api/imports").json()[0]["error"] == "Piper unavailable"
    assert client.post(f"/api/imports/{package_id}/retry").json()["status"] == "pending"
    assert client.post("/api/imports/coach/next").json()["package_id"] == package_id
    assert (
        client.post(
            f"/api/imports/{package_id}/result", json={"result_id": package_id, "references": valid}
        ).json()["added_count"]
        == 5
    )


def test_external_agent_supplies_complete_references_without_coach(
    client: TestClient, package: Package
) -> None:
    supplied = {"package": package, "references": references(package)}
    response = client.post("/api/imports", json=supplied)
    assert response.status_code == 202
    assert response.json()["status"] == "added"
    assert response.json()["added_count"] == 5
    assert client.post("/api/imports", json=supplied).json() == response.json()
    assert client.post("/api/imports/coach/next").json() is None
    card = client.post("/api/practice/daily", json={}).json()["card"]
    assert client.get(card["answer"]["reference_url"]).content.startswith(b"RIFF")
    supplied["references"]["meeting-4"] = "invalid"
    assert client.post("/api/imports", json=supplied).status_code == 409


def test_external_bad_reference_rejects_request_without_pending_work(
    client: TestClient, package: Package
) -> None:
    supplied = references(package)
    supplied["meeting-2"] = "broken"
    response = client.post("/api/imports", json={"package": package, "references": supplied})
    assert response.status_code == 409
    assert "meeting-2" in response.text
    assert client.get("/api/imports").json() == []


def silent_wav(size: int) -> str:
    output = io.BytesIO()
    with wave.open(output, "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(22_050)
        wav.writeframes(b"\x00" * (size - 44))
    return base64.b64encode(output.getvalue()).decode()


def test_reference_budget_fits_transport_and_rejects_oversized_batch(client: TestClient) -> None:
    single = make_package(lessons=[lesson("unit-1", "Meetings", [phrase(1)])])
    near_limit = silent_wav(8 * 1024 * 1024)
    accepted = client.post(
        "/api/imports",
        json={"package": single, "references": {"meeting-1": near_limit}},
    )
    assert accepted.status_code == 202
    assert accepted.json()["status"] == "added"

    pair = make_package(
        "over-budget-package",
        [lesson("unit-1", "Meetings", [phrase(1, item_id="first"), phrase(2, item_id="second")])],
        "over-budget-source",
    )
    five_mib = silent_wav(5 * 1024 * 1024)
    rejected = client.post(
        "/api/imports",
        json={"package": pair, "references": {"first": five_mib, "second": five_mib}},
    )
    assert rejected.status_code == 409
    assert "9 MiB batch audio budget" in rejected.text
    assert "split the package" in rejected.text
    assert {row["package_id"] for row in client.get("/api/imports").json()} == {
        "course-meetings-v1"
    }


@pytest.mark.parametrize(
    ("change", "location"),
    [
        (lambda p: p["lessons"][0]["phrases"][2].update(text=""), ["phrases", 2, "text"]),
        (
            lambda p: p["lessons"][0]["phrases"][1].update(recall_hints=["far"]),
            ["phrases", 1, "recall_hints"],
        ),
        (
            lambda p: p["lessons"][0]["phrases"][1].update(recall_hints=["Far", "far"]),
            ["phrases", 1, "recall_hints"],
        ),
        (
            lambda p: p["lessons"][0]["phrases"][0].update(
                recall_hints=["Complete phrase number 1.", "other"]
            ),
            ["phrases", 0],
        ),
        (
            lambda p: p["lessons"][0]["phrases"][0].update(
                recall_hints=["complete PHRASE, number 1", "other"]
            ),
            ["phrases", 0],
        ),
        (
            lambda p: p["lessons"][0]["phrases"][3].update(usage_contexts=[]),
            ["phrases", 3, "usage_contexts"],
        ),
        (
            lambda p: p["lessons"][0]["phrases"][0].pop("recall_hints"),
            ["phrases", 0, "recall_hints"],
        ),
        (lambda p: p["lessons"][0].pop("lesson_id"), ["lesson_id"]),
        (
            lambda p: p["lessons"][0]["phrases"][0].update(answer="old field"),
            ["phrases", 0, "answer"],
        ),
    ],
)
def test_validation_identifies_the_item_and_field(
    client: TestClient, package: Package, change, location
) -> None:
    change(package)
    response = client.post("/api/imports", json={"package": package})
    assert response.status_code == 422
    assert response.json()["detail"][0]["loc"] == ["body", "package", "lessons", 0, *location]
    assert client.get("/api/imports").json() == []


def test_earlier_format_and_duplicate_identities_are_rejected(
    client: TestClient, package: Package
) -> None:
    package["version"] = 1
    assert client.post("/api/imports", json={"package": package}).status_code == 422
    package["version"] = 2
    duplicate_lessons = make_package(
        lessons=[lesson("unit-1", "A", [phrase(1)]), lesson("unit-1", "B", [phrase(2)])]
    )
    response = client.post("/api/imports", json={"package": duplicate_lessons})
    assert response.status_code == 422
    assert "lesson_id values must be unique" in response.text
    duplicate_items = make_package(
        lessons=[lesson("unit-1", "A", [phrase(1)]), lesson("unit-2", "B", [phrase(1)])]
    )
    response = client.post("/api/imports", json={"package": duplicate_items})
    assert "item_id values must be unique" in response.text
    oversized = make_package(
        lessons=[
            lesson(f"unit-{block}", f"Unit {block}", [phrase(block * 30 + i) for i in range(30)])
            for block in range(2)
        ]
    )
    response = client.post("/api/imports", json={"package": oversized})
    assert response.status_code == 422
    assert "at most 50 phrases" in response.text


def test_mixed_lessons_publish_together_and_reuse_lesson_identity(client: TestClient) -> None:
    first = make_package(
        lessons=[
            lesson("small-talk", "Small talk", [phrase(1), phrase(2)]),
            lesson("meetings", "Meetings", [phrase(3)]),
        ]
    )
    assert publish(client, first)["added_count"] == 3
    later = make_package("course-meetings-v2", [lesson("small-talk", "Small talk", [phrase(4)])])
    assert publish(client, later)["added_count"] == 1
    lessons = client.get("/api/lessons").json()
    assert [(item["title"], item["phrase_count"]) for item in lessons["items"]] == [
        ("Small talk", 3),
        ("Meetings", 1),
    ]
    renamed = make_package("course-meetings-v3", [lesson("meetings", "Renamed", [phrase(5)])])
    response = client.post("/api/imports", json={"package": renamed})
    assert response.status_code == 409
    assert "lessons.meetings" in response.text and "“Meetings”" in response.text
    assert client.get("/api/imports/course-meetings-v3").status_code == 404
    assert table_count(client.app.state.database, "phrases") == 4


def test_source_item_conflict_names_existing_phrase_and_deletion(
    client: TestClient, package: Package
) -> None:
    publish(client, package)
    other = make_package("other-package", [lesson("unit-9", "Other", [phrase(3)])])
    response = client.post("/api/imports", json={"package": other})
    assert response.status_code == 409
    assert "phrases.meeting-3" in response.text
    assert "“Complete phrase number 3.”" in response.text
    assert "Delete that phrase first" in response.text
    assert table_count(client.app.state.database, "lessons") == 1


class ClientCoachApi:
    """Routes Coach's API calls to the in-process app with urllib's error behavior."""

    def __init__(self, client: TestClient):
        self.client = client

    def post(self, path: str, payload: dict[str, object] | None = None) -> Any:
        response = self.client.post(f"/api/imports/{path}", json=payload or {})
        if response.status_code >= 400:
            raise urllib.error.HTTPError(
                path, response.status_code, "Conflict", Message(), io.BytesIO(response.content)
            )
        return response.json()


def test_coach_reports_the_server_explanation_for_a_conflict(
    client: TestClient, package: Package
) -> None:
    assert client.post("/api/imports", json={"package": package}).status_code == 202
    publish(client, make_package("rival-package", [lesson("unit-1", "Meetings", [phrase(5)])]))
    assert process_once(ClientCoachApi(client), PiperSpeech(None, fake=True))
    status = client.get(f"/api/imports/{package['package_id']}").json()
    assert status["status"] == "error"
    assert "phrases.meeting-5" in status["error"]
    assert "“Complete phrase number 5.”" in status["error"]
    assert "Delete that phrase first" in status["error"]


def test_same_lesson_and_item_ids_under_another_source_are_separate(
    client: TestClient, package: Package
) -> None:
    publish(client, package)
    other = make_package(
        "other-source-package",
        [lesson("unit-1", "Different title", [phrase(1, text="Another source wording.")])],
        "other-source",
    )
    assert publish(client, other)["added_count"] == 1
    titles = [item["title"] for item in client.get("/api/lessons").json()["items"]]
    assert titles == ["Meetings", "Different title"]
    assert table_count(client.app.state.database, "phrases") == 6


def test_conflict_appearing_while_coach_prepares_fails_without_partial_publication(
    client: TestClient, package: Package
) -> None:
    assert client.post("/api/imports", json={"package": package}).status_code == 202
    work = client.post("/api/imports/coach/next").json()
    rival = make_package("rival-package", [lesson("unit-1", "Meetings", [phrase(5)])])
    publish(client, rival)
    response = client.post(
        f"/api/imports/{work['package_id']}/result",
        json={"result_id": work["package_id"], "references": references(package)},
    )
    assert response.status_code == 409
    assert "phrases.meeting-5" in response.text
    assert table_count(client.app.state.database, "phrases") == 1
    assert client.get(f"/api/imports/{package['package_id']}").json()["status"] == "preparing"


def test_reference_write_failure_rolls_back_phrases_lessons_and_files(
    tmp_path: Path, package: Package, monkeypatch: pytest.MonkeyPatch
) -> None:
    importer = Importer(Database(tmp_path))
    parsed = PhrasePackage.model_validate(package)
    importer.submit(parsed)
    importer.next_work()
    original = publication.write_private
    calls = 0

    def fail_second(path: Path, content: bytes) -> None:
        nonlocal calls
        calls += 1
        if calls == 2:
            raise OSError("disk failed")
        original(path, content)

    monkeypatch.setattr(publication, "write_private", fail_second)
    with pytest.raises(OSError, match="disk failed"):
        importer.publish(parsed.package_id, parsed.package_id, references(package))
    assert table_count(importer.database, "phrases") == 0
    assert table_count(importer.database, "lessons") == 0
    assert list((tmp_path / "references").iterdir()) == []
    assert importer.status(parsed.package_id)["status"] == "preparing"


def test_case_only_item_ids_keep_distinct_audio(tmp_path: Path) -> None:
    package = make_package(
        lessons=[
            lesson(
                "unit-1",
                "Meetings",
                [phrase(1, item_id="A", text="Upper answer."), phrase(2, item_id="a")],
            )
        ]
    )
    importer = Importer(Database(tmp_path))
    parsed = PhrasePackage.model_validate(package)
    importer.submit(parsed)
    importer.next_work()
    importer.publish(parsed.package_id, parsed.package_id, references(package))
    with importer.database.read() as connection:
        rows = connection.execute(
            "SELECT id, item_id, reference_path FROM phrases ORDER BY item_id"
        ).fetchall()
    assert len({row["reference_path"] for row in rows}) == 2
    assert len({row["id"] for row in rows}) == 2
    assert all(Path(row["reference_path"]).read_bytes().startswith(b"RIFF") for row in rows)


def test_coach_prepares_every_lesson_through_the_api(client: TestClient) -> None:
    package = make_package(
        lessons=[lesson("a", "First", [phrase(1)]), lesson("b", "Second", [phrase(2), phrase(3)])]
    )
    assert prepare_with_coach(client, package)["added_count"] == 3
    assert len(phrases_of(package)) == 3


def test_documented_example_imports_three_lessons(client: TestClient) -> None:
    example = json.loads(Path("examples/meeting-phrases.json").read_text())
    assert publish(client, example)["added_count"] == 6
    titles = [item["title"] for item in client.get("/api/lessons").json()["items"]]
    assert titles == ["Running a meeting", "Scheduling", "Small talk"]
    found = client.get("/api/phrases", params={"query": "progress update"}).json()["items"]
    assert [item["text"] for item in found] == ["So far, so good."]
