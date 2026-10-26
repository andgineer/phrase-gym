import asyncio
from pathlib import Path

import pytest

from phrase_gym.contracts import PhrasePackage
from phrase_gym.importer import ImportConflict, Importer
from phrase_gym.speech import PiperSpeech
from phrase_gym.storage import Database


def test_failure_is_atomic_and_explicit_retry_adds_full_batch(
    tmp_path: Path, package: dict[str, object]
) -> None:
    async def scenario() -> None:
        importer = Importer(Database(tmp_path), PiperSpeech(None, fake=True, fake_failures=1))
        parsed = PhrasePackage.model_validate(package)
        assert importer.submit(parsed)["status"] == "preparing"
        await asyncio.gather(*importer.tasks)
        failed = importer.status(parsed.package_id)
        assert failed["status"] == "error"
        assert "retry" in str(failed["error"])
        with importer.database.read() as connection:
            assert connection.execute("SELECT COUNT(*) FROM cards").fetchone()[0] == 0
        assert importer.retry(parsed.package_id)["status"] == "preparing"
        await asyncio.gather(*importer.tasks)
        assert importer.status(parsed.package_id)["added_count"] == 5
        assert importer.submit(parsed)["added_count"] == 5

    asyncio.run(scenario())


def test_busy_and_source_identity_conflicts_do_not_overwrite(
    tmp_path: Path, package: dict[str, object]
) -> None:
    async def scenario() -> None:
        importer = Importer(Database(tmp_path), PiperSpeech(None, fake=True))
        first = PhrasePackage.model_validate(package)
        importer.submit(first)
        other = first.model_copy(update={"package_id": "other-package"})
        with pytest.raises(ImportConflict, match="preparing"):
            importer.submit(other)
        await asyncio.gather(*importer.tasks)
        with pytest.raises(ImportConflict, match="Source items already exist"):
            importer.submit(other)

    asyncio.run(scenario())


def test_validation_identifies_bad_item_and_unknown_version(
    client: object, package: dict[str, object]
) -> None:
    package["cards"][2]["answer"] = ""
    response = client.post("/api/imports", json={"package": package})
    assert response.status_code == 422
    assert response.json()["detail"][0]["loc"] == ["body", "package", "cards", 2, "answer"]
    package["version"] = 2
    response = client.post("/api/imports", json={"package": package})
    assert response.status_code == 422
