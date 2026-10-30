from datetime import UTC, datetime, timedelta
from pathlib import Path
from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

import phrase_gym.server.file_cleanup as file_cleanup
from phrase_gym.server.app import create_app
from phrase_gym.server.deletion import PhraseDeletion
from phrase_gym.server.file_cleanup import FileCleanup
from phrase_gym.server.storage import Database
from phrase_gym.server.study import StudyService
from tests.phrase_packages import Package, lesson, make_package, phrase, publish, upload_take


def rows(database: Database, sql: str, *parameters: object) -> list[tuple[object, ...]]:
    with database.read() as connection:
        return [tuple(row) for row in connection.execute(sql, parameters).fetchall()]


def phrase_files(database: Database, phrase_id: str) -> list[Path]:
    reference = rows(database, "SELECT reference_path FROM phrases WHERE id=?", phrase_id)
    takes = rows(database, "SELECT path FROM recordings WHERE phrase_id=?", phrase_id)
    return [Path(str(row[0])) for row in reference + takes]


@pytest.fixture
def opened(client: TestClient, package: Package, tmp_path: Path) -> dict[str, object]:
    publish(client, package)
    card = client.post("/api/study/current").json()["card"]
    assert upload_take(client, card, str(uuid4())).status_code == 200
    assert upload_take(client, card, str(uuid4())).status_code == 200
    return card


def test_delete_removes_phrase_reference_takes_and_open_view(
    client: TestClient, opened: dict[str, object], tmp_path: Path
) -> None:
    database = client.app.state.database
    files = phrase_files(database, str(opened["id"]))
    assert len(files) == 3 and all(path.is_file() for path in files)
    interrupted = tmp_path / "recordings" / str(opened["id"]) / "interrupted.pending"
    interrupted.write_bytes(b"partial upload")
    result = client.delete(f"/api/phrases/{opened['id']}")
    assert result.json() == {
        "deleted_text": "Complete phrase number 1.",
        "cleanup": {"pending": 0, "error": None},
    }
    assert not any(path.exists() for path in files)
    assert not (tmp_path / "recordings" / str(opened["id"])).exists()
    for table in ("phrases", "attempts", "recordings"):
        key = "id" if table == "phrases" else "phrase_id"
        assert rows(database, f"SELECT * FROM {table} WHERE {key}=?", opened["id"]) == []
    completion = client.post(
        f"/api/attempts/{opened['attempt_id']}/complete", json={"operation_id": str(uuid4())}
    )
    assert completion.status_code == 404
    assert client.post("/api/study/current").json()["card"]["text"] == "Complete phrase number 2."
    assert client.get(f"/api/phrases/{opened['id']}").status_code == 404
    assert client.delete(f"/api/phrases/{opened['id']}").status_code == 404
    found = client.get("/api/phrases", params={"query": "phrase number"}).json()["items"]
    assert [item["text"] for item in found][0] == "Complete phrase number 2."
    assert len(found) == 4
    assert client.get("/api/lessons").json()["items"][0]["phrase_count"] == 4


def test_late_upload_for_deleted_phrase_fails_without_orphan(
    client: TestClient, opened: dict[str, object], tmp_path: Path
) -> None:
    mismatched = upload_take(client, {**opened, "attempt_id": "another-attempt"}, str(uuid4()))
    assert mismatched.status_code == 409
    client.delete(f"/api/phrases/{opened['id']}")
    late = upload_take(client, opened, str(uuid4()))
    assert late.status_code == 410
    assert "phrase was deleted" in late.json()["detail"]
    assert "Discard" in late.json()["detail"]
    assert not (tmp_path / "recordings" / str(opened["id"])).exists()
    database = client.app.state.database
    assert rows(database, "SELECT * FROM phrases WHERE id=?", opened["id"]) == []
    assert rows(database, "SELECT * FROM recordings WHERE phrase_id=?", opened["id"]) == []


def test_reviewed_phrase_deletes_with_its_review_history(
    client: TestClient, package: Package, tmp_path: Path
) -> None:
    package["lessons"] = [lesson("unit-1", "Meetings", [phrase(1)])]
    publish(client, package)
    instant = datetime(2026, 10, 28, 10, tzinfo=UTC)
    study = StudyService(Database(tmp_path), lambda: instant)
    first = study.current().card
    assert first
    study.complete(first.attempt_id, str(uuid4()))
    instant += timedelta(days=1)
    review = study.current().card
    assert review
    study.reveal(review.attempt_id, "silent")
    study.complete(review.attempt_id, str(uuid4()), "recalled", "silent")
    assert client.delete(f"/api/phrases/{first.id}").status_code == 200
    assert rows(client.app.state.database, "SELECT * FROM reviews") == []


def test_unlink_failure_stays_visible_and_retry_succeeds_without_restoring(
    client: TestClient,
    opened: dict[str, object],
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    database = client.app.state.database
    reference = phrase_files(database, str(opened["id"]))[0]
    original = file_cleanup.remove_owned_path

    def refuse_reference(path: Path) -> None:
        if path == reference.resolve():
            raise PermissionError("permission denied")
        original(path)

    monkeypatch.setattr(file_cleanup, "remove_owned_path", refuse_reference)
    result = client.delete(f"/api/phrases/{opened['id']}").json()
    assert result["cleanup"]["pending"] == 1
    assert "permission denied" in result["cleanup"]["error"]
    assert "retry cleanup" in result["cleanup"]["error"]
    assert reference.is_file()
    assert rows(database, "SELECT path FROM cleanup_paths") == [(str(reference),)]
    assert client.get(f"/api/phrases/{opened['id']}").status_code == 404

    with TestClient(create_app()) as restarted:
        assert restarted.get("/api/cleanup").json()["pending"] == 1
        assert "permission denied" in restarted.get("/api/cleanup").json()["error"]
        monkeypatch.setattr(file_cleanup, "remove_owned_path", original)
        assert restarted.post("/api/cleanup/retry").json() == {"pending": 0, "error": None}
        assert restarted.get(f"/api/phrases/{opened['id']}").status_code == 404
    assert not reference.exists()


def test_restart_after_deletion_commit_before_file_cleanup(
    client: TestClient, opened: dict[str, object], tmp_path: Path
) -> None:
    database = Database(tmp_path)
    files = phrase_files(database, str(opened["id"]))
    deletion = PhraseDeletion(database, FileCleanup(database))
    assert deletion.commit(str(opened["id"])) == "Complete phrase number 1."
    assert all(path.is_file() for path in files)
    pending = {str(row[0]) for row in rows(database, "SELECT path FROM cleanup_paths")}
    assert pending == {str(files[0]), str(tmp_path / "recordings" / str(opened["id"]))}
    with TestClient(create_app()) as restarted:
        assert restarted.get("/api/cleanup").json() == {"pending": 0, "error": None}
        assert restarted.get(f"/api/phrases/{opened['id']}").status_code == 404
    assert not any(path.exists() for path in files)


def test_replacement_import_uses_fresh_identity_and_paths(
    client: TestClient,
    opened: dict[str, object],
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    database = client.app.state.database
    old_reference = phrase_files(database, str(opened["id"]))[0]

    def refuse(_path: Path) -> None:
        raise PermissionError("still in use")

    monkeypatch.setattr(file_cleanup, "remove_owned_path", refuse)
    client.delete(f"/api/phrases/{opened['id']}")
    replacement = make_package(
        "course-meetings-v2",
        [lesson("unit-1", "Meetings", [phrase(1, text="Corrected phrase number 1.")])],
    )
    publish(client, replacement)
    new = rows(database, "SELECT id, reference_path FROM phrases WHERE item_id='meeting-1'")[0]
    assert new[0] != opened["id"]
    assert new[1] != str(old_reference)
    pending = {str(row[0]) for row in rows(database, "SELECT path FROM cleanup_paths")}
    assert str(old_reference) in pending and str(new[1]) not in pending


def test_orphaned_take_file_does_not_block_folder_cleanup(
    client: TestClient, opened: dict[str, object], tmp_path: Path
) -> None:
    folder = tmp_path / "recordings" / str(opened["id"])
    (folder / "renamed-before-crash").write_bytes(b"take without a database row")
    result = client.delete(f"/api/phrases/{opened['id']}").json()
    assert result["cleanup"] == {"pending": 0, "error": None}
    assert not folder.exists()


@pytest.mark.parametrize("field", ["id", "phrase_id", "attempt_id"])
@pytest.mark.parametrize(
    "value",
    [
        "../../references/other.wav",
        "nested/take-1234",
        "take.with.dots",
        "..secret1",
        "ABCDEFGH-1234",
    ],
)
def test_unsafe_take_identities_are_rejected(
    client: TestClient, opened: dict[str, object], field: str, value: str
) -> None:
    identities = {"id": str(uuid4()), "phrase_id": opened["id"], "attempt_id": opened["attempt_id"]}
    identities[field] = value
    card = {"id": identities["phrase_id"], "attempt_id": identities["attempt_id"]}
    assert upload_take(client, card, str(identities["id"])).status_code == 422


def test_take_identity_cannot_overwrite_or_delete_another_reference(
    client: TestClient, package: Package, tmp_path: Path
) -> None:
    publish(client, package)
    card = client.post("/api/study/current").json()["card"]
    database = client.app.state.database
    other = rows(database, "SELECT id, reference_path FROM phrases WHERE id!=? LIMIT 1", card["id"])
    other_id, other_reference = str(other[0][0]), Path(str(other[0][1]))
    traversal = f"../../references/{other_id}"
    assert upload_take(client, card, traversal, b"not a reference").status_code == 422
    assert client.delete(f"/api/phrases/{card['id']}").status_code == 200
    assert other_reference.read_bytes().startswith(b"RIFF")
    assert client.get(f"/api/phrases/{other_id}/reference").content.startswith(b"RIFF")


def test_cleanup_refuses_a_recorded_folder_outside_phrase_recordings(
    client: TestClient, tmp_path: Path
) -> None:
    keep = tmp_path / "references"
    keep.mkdir(exist_ok=True)
    (keep / "kept.wav").write_bytes(b"RIFF")
    with client.app.state.database.write() as connection:
        connection.execute(
            "INSERT INTO cleanup_paths (path, phrase_id, recorded_at) VALUES (?,?,?)",
            (str(keep), "unknown-phrase", "2026-10-30T00:00:00+00:00"),
        )
    status = client.post("/api/cleanup/retry").json()
    assert (
        status["pending"] == 1 and "not the deleted phrase's recordings folder" in status["error"]
    )
    assert (keep / "kept.wav").is_file()


def record_cleanup(client: TestClient, path: Path, phrase_id: str) -> None:
    with client.app.state.database.write() as connection:
        connection.execute(
            "INSERT INTO cleanup_paths (path, phrase_id, recorded_at) VALUES (?,?,?)",
            (str(path), phrase_id, "2026-10-30T00:00:00+00:00"),
        )


def test_case_variant_take_ids_cannot_share_a_file(
    client: TestClient, opened: dict[str, object]
) -> None:
    assert upload_take(client, opened, "abcdefgh-1234", b"first take").status_code == 200
    assert upload_take(client, opened, "ABCDEFGH-1234", b"second take").status_code == 422
    assert client.get("/api/recordings/abcdefgh-1234/audio").content == b"first take"


@pytest.mark.parametrize("owner", ["matching", "other-deleted-phrase"])
def test_cleanup_keeps_a_folder_whose_phrase_still_exists_or_differs(
    client: TestClient, opened: dict[str, object], tmp_path: Path, owner: str
) -> None:
    folder = tmp_path / "recordings" / str(opened["id"])
    record_cleanup(client, folder, str(opened["id"]) if owner == "matching" else owner)
    status = client.post("/api/cleanup/retry").json()
    assert status["pending"] == 1
    expected = "still exists" if owner == "matching" else "not the deleted phrase's"
    assert expected in status["error"]
    assert len(list(folder.iterdir())) == 2


def test_cleanup_refuses_a_symlinked_folder_without_following_it(
    client: TestClient, opened: dict[str, object], tmp_path: Path
) -> None:
    target = tmp_path / "recordings" / str(opened["id"])
    link = tmp_path / "recordings" / "deleted-phrase-0001"
    link.symlink_to(target, target_is_directory=True)
    record_cleanup(client, link, "deleted-phrase-0001")
    status = client.post("/api/cleanup/retry").json()
    assert status["pending"] == 1 and "symbolic link" in status["error"]
    assert link.is_symlink() and len(list(target.iterdir())) == 2
