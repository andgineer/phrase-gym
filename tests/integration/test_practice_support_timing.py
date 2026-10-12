from datetime import timedelta

import pytest

from tests.integration.helpers import NOW
from tests.integration.practice_helpers import apply, session_fixture
from tests.integration.session_helpers import start


@pytest.mark.parametrize("kind", ["answer", "explain", "reference", "capture_complete"])
@pytest.mark.parametrize("expired", [False, True])
def test_recent_help_uses_actual_exposure_time(client, package, kind, expired):
    session = session_fixture(client, package)
    _, result = apply(client, session, "present")
    if kind == "capture_complete":
        _, result = apply(client, result["session"], "respond", modality="recorded")
    revealed_at = NOW + timedelta(minutes=10)
    client.app.state.practice.clock = lambda: revealed_at
    _, result = apply(client, result["session"], kind, occurred_at=revealed_at.isoformat())
    original = result["session"]["attempts"][0]["original"]
    reviewed_at = revealed_at + timedelta(minutes=6 if expired else 0)
    client.app.state.practice.clock = lambda: reviewed_at
    client.app.state.sessions.clock = lambda: reviewed_at
    response = start(client, takeover_session_id=session["id"])
    assert response.status_code == 200, response.text
    next_session = response.json()
    assert (
        next_session["manifest"]["targets"][0]["phrase"]["id"]
        == session["manifest"]["targets"][0]["phrase"]["id"]
    )
    _, result = apply(client, next_session, "present", occurred_at=reviewed_at.isoformat())
    assert result["session"]["attempts"][0]["supported"] is not expired
    _, result = apply(
        client, result["session"], "rate", rating="recalled", occurred_at=reviewed_at.isoformat()
    )
    assert result["receipts"][0]["eligibility_reason"] == (
        "eligible" if expired else "supported_learning_check"
    )
    previous = client.get(f"/api/v1/sessions/{session['id']}").json()
    assert previous["attempts"][0]["original"] == original
