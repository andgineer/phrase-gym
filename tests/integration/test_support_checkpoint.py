from datetime import timedelta
from uuid import uuid4

import pytest

from tests.integration.helpers import NOW
from tests.integration.practice_helpers import apply, clocks, session_fixture


@pytest.mark.parametrize("kind", ["lookup", "answer", "explain", "reference", "capture_complete"])
def test_checkpoint_support_uses_exposure_time_not_finalization(client, package, kind):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    state = result["session"]
    target = state["attempts"][0]["target_id"]
    exposed = NOW + timedelta(minutes=1)
    clocks(client, exposed)
    if kind == "lookup":
        response = client.post(
            "/api/v1/exposures",
            json={
                "id": str(uuid4()),
                "target_id": target,
                "revision": state["attempts"][0]["revision"],
                "language": "en",
                "kind": "reference",
            },
        )
        assert response.status_code == 200
    else:
        if kind == "capture_complete":
            _, result = apply(client, state, "respond", occurred_at=exposed.isoformat())
            state = result["session"]
        _, result = apply(client, state, kind, occurred_at=exposed.isoformat())
        state = result["session"]
    clocks(client, NOW + timedelta(minutes=6))
    _, result = apply(
        client,
        state,
        "rate",
        rating="recalled",
        defer_repairs=True,
        occurred_at=(NOW + timedelta(minutes=6)).isoformat(),
    )
    snapshot = client.get(f"/api/v1/sessions/{session['id']}").json()
    assert snapshot["support_exposures"][target] == [exposed.isoformat().replace("+00:00", "Z")]
    assert snapshot["support_exposures"] == result["session"]["support_exposures"]
    assert all(not times for id, times in snapshot["support_exposures"].items() if id != target)
