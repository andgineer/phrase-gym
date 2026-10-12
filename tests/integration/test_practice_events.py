from datetime import datetime, timedelta
from uuid import uuid4

from sqlalchemy import func, select

from phrase_gym.storage import Database
from phrase_gym.tables import ScheduleRow
from phrase_gym.practice.tables import ReviewReceiptRow
from tests.integration.helpers import NOW, publish
from tests.integration.practice_helpers import apply, session_fixture
from tests.integration.session_helpers import start


def test_eligible_rating_idempotent_original_snapshot_and_completion(client, package, config):
    session = session_fixture(client, package)
    _, receipt = apply(client, session, "present")
    session = receipt["session"]
    _, receipt = apply(client, session, "answer")
    session = receipt["session"]
    assert not session["attempts"][0]["original"]["supported"]
    _, receipt = apply(client, session, "respond", modality="recorded")
    session = receipt["session"]
    assert session["attempts"][0]["original"]["modality"] == "self_report"
    rated, result = apply(client, session, "rate", rating="with_effort")
    assert result["receipts"][0]["schedule_changed"]
    assert result["receipts"][0]["eligibility_reason"] == "eligible"
    assert datetime.fromisoformat(result["receipts"][0]["due_at"]) > NOW
    path = f"/api/v1/sessions/{session['id']}/events"
    assert client.post(path, json={"events": [rated]}).json()["receipts"] == result["receipts"]
    assert client.post(path, json={"events": [{**rated, "rating": "recalled"}]}).status_code == 409
    session = result["session"]
    for _ in range(2):
        _, result = apply(client, session, "present")
        _, result = apply(client, result["session"], "rate", rating="recalled")
        session = result["session"]
    assert session["status"] == "completed"
    assert client.get("/api/v1/sessions/current").json() is None
    database = Database(config.data_dir)
    try:
        with database.read() as transaction:
            assert transaction.scalar(select(func.count()).select_from(ReviewReceiptRow)) == 3
            assert all(
                row.version == 1 and row.memory_state
                for row in transaction.scalars(select(ScheduleRow))
            )
    finally:
        database.close()


def test_help_before_response_is_not_recall_and_has_learning_check(client, package):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    _, result = apply(client, result["session"], "explain")
    _, result = apply(client, result["session"], "respond", modality="recorded")
    _, result = apply(client, result["session"], "rate", rating="recalled")
    attempt = result["session"]["attempts"][0]
    assert attempt["original"]["supported"]
    assert attempt["rating"] == "recalled"
    assert result["receipts"][0]["eligibility_reason"] == "supported_learning_check"
    assert datetime.fromisoformat(result["receipts"][0]["due_at"]) == NOW + timedelta(days=1)


def test_sequence_gap_rolls_back_batch_and_invalid_time_is_adjusted(client, package, config):
    session = session_fixture(client, package)
    present = {
        "id": str(uuid4()),
        "sequence": 1,
        "step": 0,
        "kind": "present",
        "occurred_at": NOW.isoformat(),
    }
    gap = {**present, "id": str(uuid4()), "sequence": 3, "kind": "answer"}
    path = f"/api/v1/sessions/{session['id']}/events"
    assert client.post(path, json={"events": [present, gap]}).status_code == 409
    current = client.get(f"/api/v1/sessions/{session['id']}").json()
    assert current["last_sequence"] == 0 and current["attempts"] == []
    _, result = apply(
        client, current, "present", occurred_at=(NOW + timedelta(days=100)).isoformat()
    )
    assert result["receipts"][0]["clock_adjusted"]
    _, result = apply(
        client,
        result["session"],
        "rate",
        rating="recalled",
        occurred_at=(NOW - timedelta(days=100)).isoformat(),
    )
    assert result["receipts"][0]["clock_adjusted"]
    assert datetime.fromisoformat(result["receipts"][0]["occurred_at"]) == NOW
    assert len(client.get(path).json()) == 2


def test_old_session_events_are_retained_without_schedule_changes(client, package):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    session = result["session"]
    replacement = start(client, takeover_session_id=session["id"])
    assert replacement.status_code == 200
    _, result = apply(client, session, "rate", rating="recalled")
    assert result["receipts"][0]["eligibility_reason"] == "closed_session"
    assert not result["receipts"][0]["schedule_changed"]
    assert result["session"]["status"] == "superseded"
    assert client.get("/api/v1/sessions/current").json()["id"] == replacement.json()["id"]


def test_stale_content_and_technical_failure_never_grade_forgetting(client, package):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    phrase = session["manifest"]["targets"][0]["phrase"]
    package["source_revision"] = "2"
    next(t for t in package["targets"] if t["source_id"] == phrase["content"]["source_id"])[
        "purpose"
    ] = "Updated purpose"
    publish(client, package)
    _, result = apply(client, result["session"], "rate", rating="not_yet")
    assert result["receipts"][0]["eligibility_reason"] == "stale_content"
    _, result = apply(client, result["session"], "present")
    _, result = apply(client, result["session"], "failure", reason="Capture interrupted")
    _, result = apply(client, result["session"], "rate", rating="not_yet")
    assert result["receipts"][0]["eligibility_reason"] == "technical_failure"
    assert not result["receipts"][0]["schedule_changed"]


def test_recent_lookup_exposure_is_preserved_when_session_starts(client, package):
    session = session_fixture(client, package)
    phrase = session["manifest"]["targets"][0]["phrase"]
    response = client.post(
        "/api/v1/exposures",
        json={
            "id": str(uuid4()),
            "target_id": phrase["id"],
            "revision": phrase["revision"],
            "language": "en",
            "kind": "explanation",
        },
    )
    assert response.status_code == 200
    _, result = apply(client, session, "present")
    _, result = apply(client, result["session"], "rate", rating="recalled")
    assert result["session"]["attempts"][0]["original"]["supported"]
    assert result["receipts"][0]["eligibility_reason"] == "supported_learning_check"


def test_lookup_between_cue_and_original_response_retains_support(client, package):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    phrase = session["manifest"]["targets"][0]["phrase"]
    assert (
        client.post(
            "/api/v1/exposures",
            json={
                "id": str(uuid4()),
                "target_id": phrase["id"],
                "revision": phrase["revision"],
                "language": "en",
                "kind": "explanation",
            },
        ).status_code
        == 200
    )
    _, result = apply(client, result["session"], "answer")
    _, result = apply(client, result["session"], "rate", rating="recalled")
    assert result["session"]["attempts"][0]["original"]["supported"]
    assert result["receipts"][0]["eligibility_reason"] == "supported_learning_check"
