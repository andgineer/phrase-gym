import shutil
from datetime import UTC, datetime, timedelta
from pathlib import Path
from uuid import uuid4

from fastapi.testclient import TestClient

from phrase_gym.server.storage import Database
from phrase_gym.server.study import StudyService
from tests.phrase_packages import Package, lesson, phrase, publish, upload_take


def single(package: Package, **changes: object) -> Package:
    package["lessons"] = [lesson("unit-1", "Meetings", [phrase(1, **changes)])]
    return package


def review_count(database: Database) -> int:
    with database.read() as connection:
        return connection.execute("SELECT COUNT(*) FROM reviews").fetchone()[0]


def test_next_day_recall_rotation_and_single_grade(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    publish(
        client,
        single(
            package,
            cues=[
                {"situation": "First situation without the answer."},
                {"situation": "Another situation without the answer."},
            ],
        ),
    )
    current_time = datetime(2026, 10, 28, 10, tzinfo=UTC)
    service = StudyService(Database(tmp_path), lambda: current_time)
    first = service.current().card
    assert first and first.kind == "study" and first.revealed
    assert service.complete(first.attempt_id, str(uuid4())).graded is False
    assert service.current().card is None
    current_time += timedelta(days=1)
    review = service.current().card
    assert review and review.kind == "review" and not review.revealed
    assert review.text == review.reference_url == ""
    assert review.cue.situation == "First situation without the answer."
    revealed = service.reveal(review.attempt_id, "spoken")
    assert revealed and revealed.text == first.text
    operation = str(uuid4())
    result = service.complete(review.attempt_id, operation, "not_yet", "silent")
    assert result and result.graded
    assert service.complete(review.attempt_id, operation, "not_yet", "silent") == result
    assert service.current().card is None
    current_time += timedelta(days=1)
    rotated = service.current().card
    assert rotated and rotated.cue.situation == "Another situation without the answer."
    with Database(tmp_path).read() as connection:
        reviews = connection.execute("SELECT response_mode FROM reviews").fetchall()
    assert [row["response_mode"] for row in reviews] == ["spoken"]


def test_skip_does_not_grade(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    publish(client, single(package))
    current_time = datetime(2026, 10, 28, 10, tzinfo=UTC)
    service = StudyService(Database(tmp_path), lambda: current_time)
    first = service.current().card
    assert first
    service.complete(first.attempt_id, str(uuid4()))
    current_time += timedelta(days=1)
    review = service.current().card
    assert review
    service.complete(review.attempt_id, str(uuid4()))
    assert service.current().card is None
    assert review_count(Database(tmp_path)) == 0
    paths = client.get("/openapi.json").json()["paths"]
    assert not [path for path in paths if "suspend" in path or "restore" in path]


def test_stopped_directory_copy_restores_history_references_and_takes(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    publish(client, single(package))
    current_time = datetime(2026, 10, 28, 10, tzinfo=UTC)
    service = StudyService(Database(tmp_path), lambda: current_time)
    card = service.current().card
    assert card
    take = b"complete saved take"
    recording_id = str(uuid4())
    assert upload_take(client, card.model_dump(), recording_id, take).status_code == 200
    service.complete(card.attempt_id, str(uuid4()))
    current_time += timedelta(days=1)
    recall = service.current().card
    assert recall
    service.reveal(recall.attempt_id, "silent")
    service.complete(recall.attempt_id, str(uuid4()), "recalled", "silent")
    backup = tmp_path.parent / f"backup-{uuid4()}"
    shutil.copytree(tmp_path, backup)
    fallback = tmp_path.parent / f"fallback-{uuid4()}"
    tmp_path.rename(fallback)
    shutil.copytree(backup, tmp_path)
    with Database(tmp_path).read() as connection:
        row = connection.execute(
            "SELECT due_at,reference_path FROM phrases WHERE id=?", (card.id,)
        ).fetchone()
        assert row["due_at"]
        assert Path(row["reference_path"]).is_file()
        assert (
            connection.execute(
                "SELECT COUNT(*) FROM reviews WHERE phrase_id=?", (card.id,)
            ).fetchone()[0]
            == 1
        )
        saved = connection.execute(
            "SELECT path FROM recordings WHERE id=?", (recording_id,)
        ).fetchone()
        assert Path(saved["path"]).read_bytes() == take


def test_free_rehearsal_and_settings_leave_schedule_unchanged(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    publish(client, single(package))
    current_time = datetime(2026, 10, 28, 10, tzinfo=UTC)
    service = StudyService(Database(tmp_path), lambda: current_time)
    card = service.current().card
    assert card
    service.complete(card.attempt_id, str(uuid4()))
    with Database(tmp_path).read() as connection:
        due_before = connection.execute(
            "SELECT due_at FROM phrases WHERE id=?", (card.id,)
        ).fetchone()[0]
    free = service.free(card.id)
    assert free and free.kind == "free"
    service.complete(free.attempt_id, str(uuid4()))
    with Database(tmp_path).read() as connection:
        assert (
            connection.execute("SELECT due_at FROM phrases WHERE id=?", (card.id,)).fetchone()[0]
            == due_before
        )
        assert connection.execute("SELECT COUNT(*) FROM reviews").fetchone()[0] == 0
    settings = client.get("/api/settings").json()
    assert settings["learning_timezone"] == "Europe/Belgrade"
    settings["learning_timezone"] = "Not/A_Timezone"
    assert client.put("/api/settings", json=settings).status_code == 422
    settings["learning_timezone"] = "Europe/London"
    assert client.put("/api/settings", json=settings).status_code == 200
    assert client.get("/api/settings").json()["learning_timezone"] == "Europe/London"


def test_timezone_change_keeps_first_recall_on_later_local_day(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    publish(client, single(package))
    current_time = datetime(2026, 10, 28, 22, 30, tzinfo=UTC)
    service = StudyService(Database(tmp_path), lambda: current_time)
    first = service.current().card
    assert first
    service.complete(first.attempt_id, str(uuid4()))
    current_time = datetime(2026, 10, 28, 23, 5, tzinfo=UTC)
    opened = service.current().card
    assert opened and opened.kind == "review"
    settings = client.get("/api/settings").json()
    settings["learning_timezone"] = "Europe/London"
    assert client.put("/api/settings", json=settings).status_code == 200
    waiting = service.current()
    assert waiting.card is None and waiting.due_count == 0
    assert waiting.next_return == "2026-10-29T00:00:00+00:00"
    current_time = datetime(2026, 10, 29, 0, 5, tzinfo=UTC)
    later = service.current().card
    assert later and later.kind == "review" and later.attempt_id != opened.attempt_id


def test_free_rehearsal_closes_active_review_without_grade(
    client: TestClient,
    package: Package,
    tmp_path: Path,
) -> None:
    publish(client, single(package))
    current_time = datetime(2026, 10, 28, 10, tzinfo=UTC)
    database = Database(tmp_path)
    service = StudyService(database, lambda: current_time)
    first = service.current().card
    assert first
    service.complete(first.attempt_id, str(uuid4()))
    current_time += timedelta(days=1)
    review = service.current().card
    assert review and review.kind == "review"
    assert client.get(f"/api/phrases/{review.id}").json()["text"] == first.text
    assert service.current().card.attempt_id == review.attempt_id
    free = service.free(review.id)
    assert free and free.kind == "free"
    assert service.current().card is None
    with database.read() as connection:
        closed = connection.execute(
            "SELECT completed_at FROM attempts WHERE id=?", (review.attempt_id,)
        ).fetchone()
        assert closed["completed_at"]
    assert review_count(database) == 0
