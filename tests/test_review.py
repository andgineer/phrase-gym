import asyncio
import base64
import shutil
import sqlite3
from datetime import UTC, datetime, timedelta
from pathlib import Path
from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

from phrase_gym.coach.speech import PiperSpeech
from phrase_gym.server.library import LibraryService
from phrase_gym.server.storage import Database
from phrase_gym.server.study import StudyService


def add_cards(client: TestClient, package: dict[str, object]) -> None:
    assert client.post("/api/imports", json={"package": package}).status_code == 202
    work = client.post("/api/imports/coach/next").json()
    speech = PiperSpeech(None, fake=True)
    references = {
        card["item_id"]: base64.b64encode(asyncio.run(speech.synthesize(card["answer"]))).decode()
        for card in work["package"]["cards"]
    }
    assert (
        client.post(
            f"/api/imports/{package['package_id']}/result",
            json={"result_id": str(package["package_id"]), "references": references},
        ).status_code
        == 200
    )


def test_next_day_recall_rotation_and_single_grade(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    package["cards"] = [package["cards"][0]]
    package["cards"][0]["cues"] = [
        {"situation": "First situation without the answer."},
        {"situation": "Another situation without the answer."},
    ]
    add_cards(client, package)
    current_time = datetime(2026, 10, 28, 10, tzinfo=UTC)
    service = StudyService(Database(tmp_path), lambda: current_time)
    first = service.current().card
    assert first and first.kind == "study" and first.revealed
    assert service.complete(first.attempt_id, str(uuid4())).graded is False
    assert service.current().card is None
    current_time += timedelta(days=1)
    review = service.current().card
    assert review and review.kind == "review" and not review.revealed
    assert review.answer == review.reference_url == ""
    assert review.cue.situation == "First situation without the answer."
    revealed = service.reveal(review.attempt_id, "spoken")
    assert revealed and revealed.answer == first.answer
    operation = str(uuid4())
    result = service.complete(review.attempt_id, operation, "not_yet", "silent")
    assert result and result.graded
    assert service.complete(review.attempt_id, operation, "not_yet", "silent") == result
    assert service.current().card is None
    current_time += timedelta(days=1)
    rotated = service.current().card
    assert rotated and rotated.cue.situation == "Another situation without the answer."
    history = client.get("/api/study/history").json()[0]
    assert len(history["reviews"]) == 1
    assert history["reviews"][0]["response_mode"] == "spoken"


def test_skip_and_suspension_do_not_grade(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    package["cards"] = [package["cards"][0]]
    add_cards(client, package)
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
    assert client.get("/api/study/history").json()[0]["reviews"] == []
    assert client.post(f"/api/cards/{first.id}/suspend").json() == {"suspended": True}
    assert client.post(f"/api/cards/{first.id}/restore").json() == {"suspended": False}


def test_stopped_directory_copy_restores_history_references_and_takes(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    package["cards"] = [package["cards"][0]]
    add_cards(client, package)
    current_time = datetime(2026, 10, 28, 10, tzinfo=UTC)
    service = StudyService(Database(tmp_path), lambda: current_time)
    card = service.current().card
    assert card
    take = b"complete saved take"
    recording_id = str(uuid4())
    assert (
        client.post(
            "/api/recordings",
            json={
                "id": recording_id,
                "card_id": card.id,
                "attempt_id": card.attempt_id,
                "captured_at": current_time.isoformat(),
                "mime_type": "audio/webm",
                "duration_ms": 700,
                "complete": True,
                "audio_base64": base64.b64encode(take).decode(),
            },
        ).status_code
        == 200
    )
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
            "SELECT due_at,reference_path FROM cards WHERE id=?", (card.id,)
        ).fetchone()
        assert row["due_at"]
        assert Path(row["reference_path"]).is_file()
        assert (
            connection.execute(
                "SELECT COUNT(*) FROM reviews WHERE card_id=?", (card.id,)
            ).fetchone()[0]
            == 1
        )
        saved = connection.execute(
            "SELECT path FROM recordings WHERE id=?", (recording_id,)
        ).fetchone()
        assert Path(saved["path"]).read_bytes() == take


def test_library_rehearsal_settings_and_exposure_leave_schedule_unchanged(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    package["cards"] = [package["cards"][0]]
    add_cards(client, package)
    current_time = datetime(2026, 10, 28, 10, tzinfo=UTC)
    service = StudyService(Database(tmp_path), lambda: current_time)
    card = service.current().card
    assert card
    service.complete(card.attempt_id, str(uuid4()))
    with Database(tmp_path).read() as connection:
        due_before = connection.execute(
            "SELECT due_at FROM cards WHERE id=?", (card.id,)
        ).fetchone()[0]
    free = service.free(card.id)
    assert free and free.kind == "free"
    service.complete(free.attempt_id, str(uuid4()))
    with Database(tmp_path).read() as connection:
        assert (
            connection.execute("SELECT due_at FROM cards WHERE id=?", (card.id,)).fetchone()[0]
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


def test_existing_study_upgrades_without_same_day_recall(tmp_path: Path) -> None:
    path = tmp_path / "phrase-gym.sqlite3"
    connection = sqlite3.connect(path)
    connection.executescript(
        "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);"
        "INSERT INTO meta VALUES ('schema_version','1');"
        "CREATE TABLE cards (id TEXT PRIMARY KEY, source_id TEXT, item_id TEXT,"
        "source_label TEXT, source_link TEXT, answer TEXT, explanation TEXT,"
        "cues_json TEXT, variants_json TEXT, detailed_explanation TEXT,"
        "reference_path TEXT, created_at TEXT, studied_at TEXT);"
    )
    connection.execute(
        "INSERT INTO cards VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
        (
            "old-card",
            "old-source",
            "old-item",
            "Old source",
            None,
            "Old answer",
            "Old help",
            '[{"situation":"An old situation."}]',
            "[]",
            None,
            str(tmp_path / "old.wav"),
            "2026-10-28T10:00:00+00:00",
            "2026-10-28T10:00:00+00:00",
        ),
    )
    connection.commit()
    connection.close()
    database = Database(tmp_path)
    with database.read() as upgraded:
        card = upgraded.execute("SELECT answer,due_at FROM cards WHERE id='old-card'").fetchone()
        assert card["answer"] == "Old answer"
        assert card["due_at"] > "2026-10-28T10:00:00+00:00"
        assert (
            upgraded.execute("SELECT value FROM meta WHERE key='schema_version'").fetchone()[0]
            == "2"
        )


def test_timezone_change_keeps_first_recall_on_later_local_day(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    package["cards"] = [package["cards"][0]]
    add_cards(client, package)
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


def test_viewing_answer_closes_active_review_without_grade(
    client: TestClient,
    package: dict[str, object],
    tmp_path: Path,
) -> None:
    package["cards"] = [package["cards"][0]]
    add_cards(client, package)
    current_time = datetime(2026, 10, 28, 10, tzinfo=UTC)
    database = Database(tmp_path)
    service = StudyService(database, lambda: current_time)
    library = LibraryService(database, lambda: current_time)
    first = service.current().card
    assert first
    service.complete(first.attempt_id, str(uuid4()))
    current_time += timedelta(days=1)
    review = service.current().card
    assert review
    assert library.expose(review.id)
    assert service.current().card is None
    with pytest.raises(ValueError, match="already completed"):
        service.complete(review.attempt_id, str(uuid4()), "recalled", "silent")
    with database.read() as connection:
        assert connection.execute("SELECT COUNT(*) FROM reviews").fetchone()[0] == 0
        closed = connection.execute(
            "SELECT completed_at FROM attempts WHERE id=?", (review.attempt_id,)
        ).fetchone()
        assert closed["completed_at"]
    current_time += timedelta(days=1)
    resumed = service.current().card
    assert resumed and resumed.kind == "review"
    free = service.free(resumed.id)
    assert free and free.kind == "free"
    assert service.current().card is None
    with database.read() as connection:
        assert connection.execute("SELECT COUNT(*) FROM reviews").fetchone()[0] == 0
