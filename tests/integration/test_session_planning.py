from datetime import datetime, timedelta
from uuid import uuid4

from sqlalchemy import func, select

from phrase_gym.storage import Database
from phrase_gym.tables import ScheduleRow, SessionRow
from tests.integration.helpers import NOW, publish
from tests.integration.session_helpers import ready_package, start


def test_plan_pins_unique_targets_cues_groups_and_media(client, package, config):
    content = ready_package(client, package)
    response = start(client)
    assert response.status_code == 200, response.text
    session = response.json()
    assert datetime.fromisoformat(session["created_at"]) == NOW
    manifest = session["manifest"]
    planned = manifest["targets"]
    assert {t["phrase"]["id"] for t in planned} == {p["id"] for p in content}
    assert len(planned) == 3  # Shared lesson/dialogue appearances remain one target.
    assert len({t["review_group_id"] for t in planned}) == 3
    assert all(t["schedule_version"] == 0 and t["eligibility"] == "new" for t in planned)
    for item in planned:
        cue = item["phrase"]["content"]["cues"][0]
        assert item["cue_id"] == cue["id"]
        assert set(item["hidden_chunks"]) == (
            set(item["phrase"]["content"]["required_chunks"]) - set(cue["visible_chunks"])
        )
    assert set(manifest["required_media_ids"]) == {
        p["content"]["reference_audio_id"] for p in content
    }
    database = Database(config.data_dir)
    try:
        with database.read() as transaction:
            assert transaction.scalar(select(func.count()).select_from(ScheduleRow)) == 0
    finally:
        database.close()


def test_due_first_future_excluded_and_selected_lesson_shares_versions(client, package, config):
    content = ready_package(client, package)
    by_source = {p["content"]["source_id"]: p for p in content}
    database = Database(config.data_dir)
    try:
        with database.write() as transaction:
            transaction.add_all(
                [
                    ScheduleRow(
                        target_id=by_source["day"]["id"], version=4, due_at=NOW.isoformat()
                    ),
                    ScheduleRow(
                        target_id=by_source["rather"]["id"],
                        version=2,
                        due_at=(NOW + timedelta(days=1)).isoformat(),
                    ),
                ]
            )
        lesson = by_source["day"]["lessons"][0]["id"]
        response = start(client, lesson_id=lesson)
        assert response.status_code == 200, response.text
        manifest = response.json()["manifest"]
        assert manifest["lesson_id"] == lesson
        assert [t["eligibility"] for t in manifest["targets"]] == ["due", "new"]
        assert manifest["targets"][0]["schedule_version"] == 4
        all_lessons = start(client, takeover_session_id=response.json()["id"]).json()
        assert len(all_lessons["manifest"]["targets"]) == 2
        assert all_lessons["manifest"]["targets"][0]["schedule_version"] == 4
        with database.read() as transaction:
            assert transaction.get(ScheduleRow, by_source["day"]["id"]).version == 4
    finally:
        database.close()


def test_no_ready_content_and_invalid_scope_do_not_create_session(client, package, config):
    publish(client, package)
    assert start(client).status_code == 409
    assert start(client, lesson_id=str(uuid4())).status_code == 404
    assert start(client, language="fr").status_code == 422
    assert client.get("/api/v1/sessions/current").json() is None
    content = ready_package(client, package)
    lesson = content[0]["lessons"][0]["id"]
    assert start(client, language="sr", lesson_id=lesson).status_code == 404
    database = Database(config.data_dir)
    try:
        with database.read() as transaction:
            assert transaction.scalar(select(func.count()).select_from(SessionRow)) == 0
    finally:
        database.close()


def test_missing_reference_excluded_and_failed_takeover_preserves_session(client, package, config):
    ready_package(client, package)
    first = start(client).json()
    assert start(client, language="sr", takeover_session_id=first["id"]).status_code == 409
    assert client.get("/api/v1/sessions/current").json() == first
    for path in (config.data_dir / "media").rglob("*"):
        if path.is_file():
            path.unlink()
    assert start(client, takeover_session_id=first["id"]).status_code == 409
    assert client.get("/api/v1/sessions/current").json() == first


def test_manifest_immutable_after_publication_and_navigation(client, package):
    ready_package(client, package)
    first = start(client).json()
    package["source_revision"] = "2"
    package["targets"][0]["purpose"] = "A revised editorial purpose."
    publish(client, package)
    client.get("/api/v1/phrases/search", params={"language": "sr"})
    current = client.get(f"/api/v1/sessions/{first['id']}").json()
    assert current == first
    assert client.get("/api/v1/sessions/current").json() == first
    replacement = start(client, takeover_session_id=first["id"]).json()
    day = next(
        t
        for t in replacement["manifest"]["targets"]
        if t["phrase"]["content"]["source_id"] == "day"
    )
    assert day["phrase"]["revision"] == 2


def test_selected_lesson_excludes_its_future_targets(client, package, config):
    content = ready_package(client, package)
    catch_up = next(p for p in content if p["content"]["source_id"] == "catch-up")
    lesson = next(lesson for lesson in catch_up["lessons"] if lesson["title"] == "Building rapport")
    database = Database(config.data_dir)
    try:
        with database.write() as transaction:
            transaction.add(
                ScheduleRow(
                    target_id=catch_up["id"],
                    version=7,
                    due_at=(NOW + timedelta(days=2)).isoformat(),
                )
            )
        planned = start(client, lesson_id=lesson["id"]).json()["manifest"]["targets"]
        assert [t["phrase"]["content"]["source_id"] for t in planned] == ["day"]
        # The complete lesson list remains independent of scheduling eligibility.
        listed = client.get(f"/api/v1/lessons/{lesson['id']}/phrases", params={"language": "en"})
        assert listed.status_code == 200, listed.text
        assert len(listed.json()["items"]) == 2
    finally:
        database.close()
