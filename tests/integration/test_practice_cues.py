from datetime import timedelta


from phrase_gym.practice.tables import ChunkHistoryRow
from phrase_gym.storage import Database
from phrase_gym.tables import ScheduleRow
from tests.integration.helpers import NOW
from tests.integration.practice_helpers import apply
from tests.integration.session_helpers import ready_package, start


def test_cue_selection_uses_persistent_hidden_chunk_history(client, package, config):
    day = package["targets"][0]
    day["cues"].append(
        {
            "id": "subject-cue",
            "prompt": day["purpose"],
            "visible_chunks": ["opening", "ending"],
            "reviewed": True,
        }
    )
    ready_package(client, package)
    first = start(client).json()
    day_index = next(
        i
        for i, t in enumerate(first["manifest"]["targets"])
        if t["phrase"]["content"]["source_id"] == "day"
    )
    for index in range(day_index + 1):
        _, result = apply(client, first, "present")
        _, result = apply(client, result["session"], "rate", rating="recalled")
        first = result["session"]
    chosen = first["manifest"]["targets"][day_index]
    database = Database(config.data_dir)
    try:
        with database.read() as transaction:
            history = transaction.get(
                ChunkHistoryRow, (str(chosen["phrase"]["id"]), chosen["hidden_chunks"][0])
            )
            assert history.presentations == 1 and history.recalls == 1
        with database.write() as transaction:
            schedule = transaction.get(ScheduleRow, chosen["phrase"]["id"])
            schedule.due_at = (NOW - timedelta(days=1)).isoformat()
        second = start(client, takeover_session_id=first["id"]).json()
        selected = next(
            t for t in second["manifest"]["targets"] if t["phrase"]["id"] == chosen["phrase"]["id"]
        )
        assert selected["cue_id"] != chosen["cue_id"]
    finally:
        database.close()
