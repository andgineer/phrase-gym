import asyncio
import base64
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

import phrase_gym.server.publication as publication
from phrase_gym.coach.speech import PiperSpeech
from phrase_gym.server.app import create_app
from phrase_gym.server.contracts import PhrasePackage
from phrase_gym.server.importer import Importer
from phrase_gym.server.publication import card_identity
from phrase_gym.server.storage import Database


def references(package: dict[str, object]) -> dict[str, str]:
    speech = PiperSpeech(None, fake=True)
    cards = package["cards"]
    assert isinstance(cards, list)
    return {
        card["item_id"]: base64.b64encode(asyncio.run(speech.synthesize(card["answer"]))).decode()
        for card in cards
    }


def test_offline_restart_retry_and_idempotent_publication(
    tmp_path: Path, package: dict[str, object], monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    with TestClient(create_app()) as client:
        accepted = client.post("/api/imports", json={"package": package})
        assert accepted.status_code == 202
        assert accepted.json()["status"] == "pending"
        assert client.post("/api/study/current").json()["total"] == 0
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
        assert client.post("/api/study/current").json()["card"]["answer"]
    with Database(tmp_path).read() as connection:
        assert connection.execute("SELECT COUNT(*) FROM cards").fetchone()[0] == 5
    assert len(list((tmp_path / "references").iterdir())) == 5


def test_interrupted_preparation_can_be_retried_without_server_restart(
    client: TestClient, package: dict[str, object]
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
    client: TestClient, package: dict[str, object]
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
    assert "meeting-3" in response.text
    with client.app.state.database.read() as connection:
        assert connection.execute("SELECT COUNT(*) FROM cards").fetchone()[0] == 0
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
    client: TestClient, package: dict[str, object]
) -> None:
    supplied = {"package": package, "references": references(package)}
    response = client.post("/api/imports", json=supplied)
    assert response.status_code == 202
    assert response.json()["status"] == "added"
    assert response.json()["added_count"] == 5
    assert client.post("/api/imports", json=supplied).json() == response.json()
    assert client.post("/api/imports/coach/next").json() is None
    study = client.post("/api/study/current").json()
    assert client.get(study["card"]["reference_url"]).content.startswith(b"RIFF")
    supplied["references"]["meeting-4"] = "invalid"
    assert client.post("/api/imports", json=supplied).status_code == 409


def test_external_bad_reference_rejects_request_without_pending_work(
    client: TestClient, package: dict[str, object]
) -> None:
    supplied = references(package)
    supplied["meeting-2"] = "broken"
    response = client.post("/api/imports", json={"package": package, "references": supplied})
    assert response.status_code == 409
    assert "meeting-2" in response.text
    assert client.get("/api/imports").json() == []


def test_validation_identifies_bad_item_and_unknown_version(
    client: TestClient, package: dict[str, object]
) -> None:
    package["cards"][2]["answer"] = ""
    response = client.post("/api/imports", json={"package": package})
    assert response.status_code == 422
    assert response.json()["detail"][0]["loc"] == ["body", "package", "cards", 2, "answer"]
    package["version"] = 2
    assert client.post("/api/imports", json={"package": package}).status_code == 422


def test_reference_write_failure_rolls_back_cards_and_files(
    tmp_path: Path, package: dict[str, object], monkeypatch: pytest.MonkeyPatch
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
    with importer.database.read() as connection:
        assert connection.execute("SELECT COUNT(*) FROM cards").fetchone()[0] == 0
    assert list((tmp_path / "references").iterdir()) == []
    assert importer.status(parsed.package_id)["status"] == "preparing"


def test_source_identity_conflict_after_publication(
    tmp_path: Path, package: dict[str, object]
) -> None:
    importer = Importer(Database(tmp_path))
    parsed = PhrasePackage.model_validate(package)
    importer.submit(parsed)
    importer.next_work()
    importer.publish(parsed.package_id, parsed.package_id, references(package))
    other = parsed.model_copy(update={"package_id": "other-package"})
    with pytest.raises(ValueError, match="meeting-1"):
        importer.submit(other)


def test_case_only_item_ids_keep_distinct_audio(tmp_path: Path, package: dict[str, object]) -> None:
    package["cards"] = [
        {
            "item_id": item,
            "answer": answer,
            "explanation": "A complete explanation.",
            "cues": [{"situation": "A sufficiently clear situation."}],
        }
        for item, answer in (("A", "Upper answer."), ("a", "Lower answer."))
    ]
    importer = Importer(Database(tmp_path))
    parsed = PhrasePackage.model_validate(package)
    importer.submit(parsed)
    importer.next_work()
    importer.publish(parsed.package_id, parsed.package_id, references(package))
    with importer.database.read() as connection:
        rows = connection.execute(
            "SELECT item_id, reference_path FROM cards ORDER BY item_id"
        ).fetchall()
    assert len({row["reference_path"] for row in rows}) == 2
    assert all(Path(row["reference_path"]).read_bytes().startswith(b"RIFF") for row in rows)
    assert card_identity(parsed.source_id, "A") != card_identity(parsed.source_id, "a")
