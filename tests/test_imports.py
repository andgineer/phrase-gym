import base64
import io
import json
import wave
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

import phrase_gym.server.publication as publication
from phrase_gym.server.import_contracts import PhrasePackage
from phrase_gym.server.importer import Importer
from phrase_gym.server.storage import Database
from tests.phrase_packages import (
    Package,
    lesson,
    make_package,
    phrase,
    publish,
    references,
)


def table_count(database: Database, table: str) -> int:
    with database.read() as connection:
        return connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]


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
    card = client.post(
        "/api/practice/reveal", json={key: card[key] for key in ("view_token", "phrase_id")}
    ).json()
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
    assert client.get(f"/api/imports/{package['package_id']}").status_code == 404


def test_package_without_references_is_rejected(client: TestClient, package: Package) -> None:
    response = client.post("/api/imports", json={"package": package})
    assert response.status_code == 422
    assert response.json()["detail"][0]["loc"] == ["body", "references"]
    assert client.get(f"/api/imports/{package['package_id']}").status_code == 404


def test_word_sense_card_accepts_three_contexts(client: TestClient) -> None:
    contexts = [
        {"situation": f"Context {i}: she was __________ to go. (=unwilling)"} for i in range(3)
    ]
    single = make_package(lessons=[lesson("words", "Words", [phrase(1, cues=contexts)])])
    assert publish(client, single)["added_count"] == 1


def test_coach_source_is_reserved_for_the_coach_field(client: TestClient) -> None:
    reserved = make_package(source_id="phrase-gym-coach")
    response = client.post(
        "/api/imports", json={"package": reserved, "references": references(reserved)}
    )
    assert response.status_code == 409
    assert "reserved for the Coach field" in response.text


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
    assert client.get("/api/imports/over-budget-package").status_code == 404


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
        (
            lambda p: p["lessons"][0]["phrases"][1].update(
                cues=[{"situation": f"Context number {i}."} for i in range(4)]
            ),
            ["phrases", 1, "cues"],
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
    response = client.post("/api/imports", json={"package": package, "references": {}})
    assert response.status_code == 422
    assert response.json()["detail"][0]["loc"] == ["body", "package", "lessons", 0, *location]
    assert client.get(f"/api/imports/{package['package_id']}").status_code == 404


def test_earlier_format_and_duplicate_identities_are_rejected(
    client: TestClient, package: Package
) -> None:
    package["version"] = 1
    assert (
        client.post("/api/imports", json={"package": package, "references": {}}).status_code == 422
    )
    package["version"] = 2
    duplicate_lessons = make_package(
        lessons=[lesson("unit-1", "A", [phrase(1)]), lesson("unit-1", "B", [phrase(2)])]
    )
    response = client.post("/api/imports", json={"package": duplicate_lessons, "references": {}})
    assert response.status_code == 422
    assert "lesson_id values must be unique" in response.text
    duplicate_items = make_package(
        lessons=[lesson("unit-1", "A", [phrase(1)]), lesson("unit-2", "B", [phrase(1)])]
    )
    response = client.post("/api/imports", json={"package": duplicate_items, "references": {}})
    assert "item_id values must be unique" in response.text
    oversized = make_package(
        lessons=[
            lesson(f"unit-{block}", f"Unit {block}", [phrase(block * 30 + i) for i in range(30)])
            for block in range(2)
        ]
    )
    response = client.post("/api/imports", json={"package": oversized, "references": {}})
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
    response = client.post(
        "/api/imports", json={"package": renamed, "references": references(renamed)}
    )
    assert response.status_code == 409
    assert "lessons.meetings" in response.text and "“Meetings”" in response.text
    assert client.get("/api/imports/course-meetings-v3").status_code == 404
    assert table_count(client.app.state.database, "phrases") == 4


def test_source_item_conflict_names_existing_phrase_and_deletion(
    client: TestClient, package: Package
) -> None:
    publish(client, package)
    other = make_package("other-package", [lesson("unit-9", "Other", [phrase(3)])])
    response = client.post("/api/imports", json={"package": other, "references": references(other)})
    assert response.status_code == 409
    assert "phrases.meeting-3" in response.text
    assert "“Complete phrase number 3.”" in response.text
    assert "Delete that phrase first" in response.text
    assert table_count(client.app.state.database, "lessons") == 1


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


def test_reference_write_failure_rolls_back_phrases_lessons_and_files(
    tmp_path: Path, package: Package, monkeypatch: pytest.MonkeyPatch
) -> None:
    importer = Importer(Database(tmp_path))
    parsed = PhrasePackage.model_validate(package)
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
        importer.submit(parsed, references(package))
    assert table_count(importer.database, "phrases") == 0
    assert table_count(importer.database, "lessons") == 0
    assert table_count(importer.database, "imports") == 0
    assert list((tmp_path / "references").iterdir()) == []


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
    importer.submit(PhrasePackage.model_validate(package), references(package))
    with importer.database.read() as connection:
        rows = connection.execute(
            "SELECT id, item_id, reference_path FROM phrases ORDER BY item_id"
        ).fetchall()
    assert len({row["reference_path"] for row in rows}) == 2
    assert len({row["id"] for row in rows}) == 2
    assert all(Path(row["reference_path"]).read_bytes().startswith(b"RIFF") for row in rows)


def test_documented_example_imports_three_lessons(client: TestClient) -> None:
    example = json.loads(Path("examples/meeting-phrases.json").read_text())
    assert publish(client, example)["added_count"] == 6
    titles = [item["title"] for item in client.get("/api/lessons").json()["items"]]
    assert titles == ["Running a meeting", "Scheduling", "Small talk"]
    found = client.get("/api/phrases", params={"query": "progress update"}).json()["items"]
    assert [item["text"] for item in found] == ["So far, so good."]
