import shutil
from pathlib import Path
from uuid import uuid4

from fastapi.testclient import TestClient

from phrase_gym.server.storage import Database
from tests.phrase_packages import Package, lesson, phrase, publish, upload_take
from tests.practice_client import Practice


def test_stopped_directory_copy_restores_practice_references_and_takes(
    client: TestClient,
    package: Package,
    tmp_path: Path,
) -> None:
    package["lessons"] = [lesson("unit-1", "Meetings", [phrase(1)])]
    publish(client, package)
    practice = Practice(client)
    card = practice.card()
    take = b"complete saved take"
    recording_id = str(uuid4())
    assert upload_take(client, card, recording_id, take).status_code == 200
    practice.finish(card, "studied")
    practice.later(days=1)
    recall = practice.card()
    practice.reveal(recall)
    practice.finish(recall, "got_it")
    backup = tmp_path.parent / f"backup-{uuid4()}"
    shutil.copytree(tmp_path, backup)
    fallback = tmp_path.parent / f"fallback-{uuid4()}"
    tmp_path.rename(fallback)
    shutil.copytree(backup, tmp_path)
    with Database(tmp_path).read() as connection:
        row = connection.execute(
            "SELECT due_at, last_review, reference_path FROM phrases WHERE id=?",
            (card["phrase_id"],),
        ).fetchone()
        assert row["due_at"] and row["last_review"]
        assert Path(row["reference_path"]).is_file()
        log = connection.execute("SELECT kind FROM practice_log ORDER BY completed_at").fetchall()
        assert [entry["kind"] for entry in log] == ["first_study", "recall"]
        saved = connection.execute(
            "SELECT path FROM recordings WHERE id=?", (recording_id,)
        ).fetchone()
        assert Path(saved["path"]).read_bytes() == take
