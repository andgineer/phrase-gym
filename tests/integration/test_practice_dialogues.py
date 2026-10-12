from uuid import uuid4

from sqlalchemy import select

from phrase_gym.practice.tables import CueReportRow
from phrase_gym.storage import Database
from phrase_gym.tables import ScheduleRow
from tests.integration.practice_helpers import apply
from tests.integration.session_helpers import ready_package, start


def test_dialogue_exposure_detour_cursor_and_repeated_target_share_schedule(
    client, package, config
):
    catch_up = package["targets"][1]
    package["dialogues"][0]["turns"] += [
        {"speaker": "Partner", "text": "Good idea. Shall we meet again next week?"},
        {"speaker": "Learner", "text": catch_up["text"], "target_source_id": catch_up["source_id"]},
        {"speaker": "Partner", "text": "See you then."},
    ]
    ready_package(client, package)
    dialogue = client.get("/api/v1/dialogues?language=en").json()["items"][0]
    session = start(client, dialogue_id=dialogue["id"]).json()
    assert [step["dialogue_turn"] for step in session["queue"]] == [1, 3]
    assert len(session["manifest"]["targets"]) == 1
    _, result = apply(client, session, "dialogue_exposure")
    _, result = apply(client, result["session"], "dialogue_begin")
    _, result = apply(client, result["session"], "present")
    _, result = apply(client, result["session"], "detour")
    assert result["session"]["queue"][0]["dialogue_turn"] is None
    assert result["session"]["queue"][1]["dialogue_turn"] == 3
    _, result = apply(client, result["session"], "rate", rating="recalled")
    assert result["receipts"][0]["eligibility_reason"] == "supported_learning_check"
    _, result = apply(client, result["session"], "present")
    _, result = apply(client, result["session"], "rate", rating="recalled")
    assert not result["receipts"][0]["schedule_changed"]
    assert result["session"]["status"] == "completed"
    assert len(result["session"]["manifest"]["dialogue"]["content"]["turns"]) == 5
    database = Database(config.data_dir)
    try:
        with database.read() as transaction:
            schedules = transaction.scalars(select(ScheduleRow)).all()
            assert len(schedules) == 1 and schedules[0].version == 1
            assert schedules[0].memory_state is None
    finally:
        database.close()


def test_explicit_cue_report_excludes_only_its_revision_without_a_grade(client, package, config):
    ready_package(client, package)
    session = start(client).json()
    _, result = apply(client, session, "present")
    _, result = apply(client, result["session"], "report", reason="This cue is ambiguous")
    assert result["session"]["attempts"][0]["rating"] is None
    replacement = start(client, takeover_session_id=session["id"]).json()
    target_id = session["manifest"]["targets"][0]["phrase"]["id"]
    assert target_id not in [t["phrase"]["id"] for t in replacement["manifest"]["targets"]]
    assert (
        start(client, dialogue_id=str(uuid4()), takeover_session_id=replacement["id"]).status_code
        == 404
    )
    database = Database(config.data_dir)
    try:
        with database.read() as transaction:
            assert len(transaction.scalars(select(CueReportRow)).all()) == 1
            assert transaction.scalars(select(ScheduleRow)).all() == []
    finally:
        database.close()
