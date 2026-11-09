import json
from uuid import uuid4

from fastapi.testclient import TestClient

from phrase_gym.server.app import create_app
from phrase_gym.server.practice_contracts import CompletionReceipt, CompletionRequest
from phrase_gym.server.schema_upgrade import STEPS, statements
from tests.phrase_packages import Package, publish, upload_take
from tests.practice_client import Practice


def test_upgrade_closes_old_study_view_and_preserves_history_retries_and_pending_takes(
    client: TestClient,
    package: Package,
) -> None:
    publish(client, package)
    practice = Practice(client)
    card = practice.card()
    legacy = {**card, "kind": "first_study", "cue": None}
    request = CompletionRequest(
        view_token=str(uuid4()),
        phrase_id=card["phrase_id"],
        completion_id=str(uuid4()),
        outcome="studied",
    )
    receipt = CompletionReceipt(
        completion_id=request.completion_id,
        phrase_id=card["phrase_id"],
        learning_date="2026-10-27",
        mode="daily",
        kind="first_study",
        rating=None,
        skipped=False,
        fsrs_updated=False,
    )
    with practice.database.write() as connection:
        connection.execute("UPDATE meta SET value='6' WHERE key='schema_version'")
        # Schema 6 still kept one column per setting.
        connection.execute("DROP TABLE settings")
        for statement in statements(STEPS[5]):
            if statement.lstrip().startswith(("CREATE TABLE settings", "INSERT INTO settings")):
                connection.execute(statement)
        connection.execute("UPDATE current_view SET kind='first_study', revealed=1, cue_index=NULL")
        connection.execute(
            "INSERT INTO operations VALUES (?, 'completion', ?, ?)",
            (request.completion_id, request.model_dump_json(), receipt.model_dump_json()),
        )
        connection.execute(
            "INSERT INTO practice_log VALUES (?, ?, ?, 'Meetings', ?, '2026-10-27', "
            "'first_study', NULL, 0, 0, 'daily')",
            (
                request.completion_id,
                card["phrase_id"],
                "Complete phrase number 1.",
                "2026-10-27T10:00:00+00:00",
            ),
        )
    with TestClient(create_app()) as restarted:
        upgraded = Practice(restarted, practice.now)
        replay = restarted.post("/api/practice/complete", json=request.model_dump())
        assert replay.json() == {"receipt": receipt.model_dump(), "replayed": True, "state": None}
        changed = {**request.model_dump(), "outcome": "got_it"}
        assert restarted.post("/api/practice/complete", json=changed).status_code == 409
        assert upgraded.complete(legacy, "studied").status_code == 409
        assert upgraded.action("reveal", legacy).status_code == 409
        assert upload_take(restarted, legacy, str(uuid4())).status_code == 200
        [take] = restarted.get(f"/api/phrases/{card['phrase_id']}/recordings").json()["items"]
        assert take["kind"] == "first_study" and take["cue"] is None
        work = restarted.get("/api/practice/facts").json()["work"]
        assert work["first_meetings"] == 1 and work["new_attempts"] == work["recalls"] == 0
        assert sum(work[key] for key in ("easy", "got_it", "effort", "not_yet")) == 0
        fresh = upgraded.card()
        assert fresh["kind"] == "new" and not fresh["revealed"] and fresh["answer"] is None
        assert fresh["view_token"] != card["view_token"] and fresh["cue"]
        assert upgraded.phrase(card["phrase_id"])["fsrs_json"] is None
        assert (
            json.loads(upgraded.rows("SELECT result_json FROM operations")[0]["result_json"])
            == receipt.model_dump()
        )
