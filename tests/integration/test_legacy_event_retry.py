import json

from phrase_gym.tables import SessionRow
from tests.integration.practice_helpers import apply, event, session_fixture


def test_new_sessions_reject_retired_queue_option(client, package):
    state = session_fixture(client, package)
    assert state["manifest"]["planner_version"] == "due-new-v3-online"
    _, response = event(client, state, "rate", rating="not_yet", defer_repairs=True)
    assert response.status_code == 422
    current = client.get(f"/api/v1/sessions/{state['id']}").json()
    assert current["last_sequence"] == 0
    schema = client.get("/openapi.json").json()["components"]["schemas"]["PracticeInput"]
    assert "defer_repairs" not in schema["properties"]
    assert "legacy_deferral" not in schema["properties"]


def test_old_queued_rating_preserves_payload_and_receipt(client, package):
    state = session_fixture(client, package)
    with client.app.state.sessions.db.write() as transaction:
        row = transaction.get(SessionRow, state["id"])
        manifest = json.loads(row.manifest)
        manifest["planner_version"] = "due-new-v2"
        row.manifest = json.dumps(manifest)
    _, presented = apply(client, state, "present")
    state = presented["session"]
    body, result = apply(client, state, "rate", rating="not_yet", defer_repairs=True)
    path = f"/api/v1/sessions/{state['id']}/events"
    repeated = client.post(path, json={"events": [body]})
    assert repeated.status_code == 200
    assert repeated.json()["receipts"] == result["receipts"]
    assert repeated.json()["session"]["last_sequence"] == 2
    assert len(repeated.json()["session"]["queue"]) == len(state["queue"])
    assert client.post(path, json={"events": [{**body, "defer_repairs": False}]}).status_code == 409


def test_old_false_option_is_canonical_on_current_sessions(client, package):
    state = session_fixture(client, package)
    body, result = apply(client, state, "present", defer_repairs=False)
    path = f"/api/v1/sessions/{state['id']}/events"
    assert client.post(path, json={"events": [body]}).json()["receipts"] == result["receipts"]
    del body["defer_repairs"]
    assert client.post(path, json={"events": [body]}).status_code == 409
