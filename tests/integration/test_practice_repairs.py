from datetime import timedelta

from phrase_gym.storage import Database
from phrase_gym.tables import ScheduleRow
from tests.integration.helpers import NOW
from tests.integration.practice_helpers import apply, session_fixture


def run_cards(client, session, slow):
    for index, rating in enumerate(["not_yet", "recalled", "recalled"]):
        now = NOW + timedelta(minutes=index * 3 if slow else 0)
        client.app.state.practice.clock = lambda: now
        _, result = apply(client, session, "present", occurred_at=now.isoformat())
        _, result = apply(
            client, result["session"], "rate", rating=rating, occurred_at=now.isoformat()
        )
        session = result["session"]
    return session


def test_fast_other_cards_defer_repair_without_waiting(client, package):
    session = run_cards(client, session_fixture(client, package), slow=False)
    assert session["status"] == "completed"
    assert len(session["queue"]) == 3
    assert session["attempts"][0]["rating"] == "not_yet"


def test_separated_repair_preserves_original_and_updates_schedule_once(client, package, config):
    session = run_cards(client, session_fixture(client, package), slow=True)
    assert session["status"] == "active" and session["cursor"] == 3
    assert session["queue"][3]["repair"]
    now = NOW + timedelta(minutes=6)
    _, result = apply(client, session, "present", occurred_at=now.isoformat())
    _, result = apply(
        client, result["session"], "rate", rating="recalled", occurred_at=now.isoformat()
    )
    assert result["receipts"][0]["eligibility_reason"] == "repair"
    assert not result["receipts"][0]["schedule_changed"]
    assert result["session"]["attempts"][0]["rating"] == "not_yet"
    database = Database(config.data_dir)
    try:
        with database.read() as transaction:
            target_id = session["attempts"][0]["target_id"]
            assert transaction.get(ScheduleRow, target_id).version == 1
    finally:
        database.close()
