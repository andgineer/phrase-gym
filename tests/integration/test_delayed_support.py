from datetime import datetime, timedelta
from uuid import uuid4

import pytest

from tests.integration.helpers import NOW
from tests.integration.practice_helpers import apply, clocks, session_fixture
from tests.integration.session_helpers import start


@pytest.mark.parametrize("source", ["lookup", "reference"])
@pytest.mark.parametrize("before_response", [False, True])
@pytest.mark.parametrize("scheduled", [False, True])
def test_delayed_receipt_support_ends_at_original_response(
    client, package, source, before_response, scheduled
):
    helper = session_fixture(client, package)
    base = NOW
    if scheduled:
        _, result = apply(client, helper, "present")
        _, result = apply(client, result["session"], "rate", rating="recalled")
        base = datetime.fromisoformat(result["receipts"][0]["due_at"]) + timedelta(seconds=1)
        clocks(client, base)
        helper = start(client, takeover_session_id=helper["id"]).json()
    _, result = apply(client, helper, "present", occurred_at=base.isoformat())
    helper = result["session"]
    clocks(client, base + timedelta(seconds=1))
    delayed = start(client, takeover_session_id=helper["id"]).json()
    target = delayed["manifest"]["targets"][0]["phrase"]
    assert target["id"] == helper["manifest"]["targets"][0]["phrase"]["id"]
    exposure_time = base + timedelta(seconds=3 if before_response else 5)
    if source == "lookup":
        clocks(client, exposure_time)
        response = client.post(
            "/api/v1/exposures",
            json={
                "id": str(uuid4()),
                "target_id": target["id"],
                "revision": target["revision"],
                "language": "en",
                "kind": "reference",
            },
        )
        assert response.status_code == 200, response.text
    else:
        clocks(client, exposure_time)
        apply(client, helper, "reference", occurred_at=exposure_time.isoformat())
    clocks(client, base + timedelta(seconds=20))
    _, result = apply(
        client, delayed, "present", occurred_at=(base + timedelta(seconds=2)).isoformat()
    )
    assert not result["session"]["attempts"][0]["supported"]
    _, result = apply(
        client, result["session"], "answer", occurred_at=(base + timedelta(seconds=4)).isoformat()
    )
    original = result["session"]["attempts"][0]["original"]
    assert original["supported"] is before_response
    assert not result["receipts"][0]["clock_adjusted"]
    body, result = apply(
        client,
        result["session"],
        "rate",
        rating="recalled",
        occurred_at=(base + timedelta(seconds=6)).isoformat(),
    )
    receipt = result["receipts"][0]
    assert result["session"]["attempts"][0]["original"] == original
    assert receipt["eligibility_reason"] == (
        ("supported" if scheduled else "supported_learning_check")
        if before_response
        else "eligible"
    )
    assert receipt["schedule_changed"] is (not before_response or not scheduled)
    assert receipt["schedule_version"] == (1 if before_response or not scheduled else 2)
    replay = client.post(f"/api/v1/sessions/{delayed['id']}/events", json={"events": [body]})
    assert replay.status_code == 200
    assert replay.json()["receipts"] == result["receipts"]


def test_delayed_dialogue_uses_script_occurrence_before_response_not_receive_time(client, package):
    session_fixture(client, package)
    dialogue = client.get("/api/v1/dialogues?language=en").json()["items"][0]
    current = client.get("/api/v1/sessions/current").json()
    session = start(client, dialogue_id=dialogue["id"], takeover_session_id=current["id"]).json()
    clocks(client, NOW + timedelta(minutes=1))
    _, result = apply(
        client, session, "dialogue_exposure", occurred_at=(NOW + timedelta(seconds=1)).isoformat()
    )
    _, result = apply(
        client,
        result["session"],
        "dialogue_begin",
        occurred_at=(NOW + timedelta(seconds=2)).isoformat(),
    )
    _, result = apply(
        client, result["session"], "present", occurred_at=(NOW + timedelta(seconds=3)).isoformat()
    )
    assert result["session"]["attempts"][0]["supported"]
    _, result = apply(
        client, result["session"], "answer", occurred_at=(NOW + timedelta(seconds=4)).isoformat()
    )
    assert result["session"]["attempts"][0]["original"]["supported"]
    _, result = apply(
        client,
        result["session"],
        "rate",
        rating="recalled",
        occurred_at=(NOW + timedelta(seconds=5)).isoformat(),
    )
    assert result["receipts"][0]["eligibility_reason"] == "supported_learning_check"


@pytest.mark.parametrize("elapsed", [timedelta(minutes=4), timedelta(minutes=6)])
def test_script_exposure_expires_for_later_target(client, package, elapsed):
    target = package["targets"][2]
    package["dialogues"][0]["turns"].append(
        {"speaker": "Learner", "text": target["text"], "target_source_id": target["source_id"]}
    )
    session_fixture(client, package)
    dialogue = client.get("/api/v1/dialogues?language=en").json()["items"][0]
    current = client.get("/api/v1/sessions/current").json()
    session = start(client, dialogue_id=dialogue["id"], takeover_session_id=current["id"]).json()
    _, result = apply(client, session, "dialogue_exposure")
    _, result = apply(client, result["session"], "dialogue_begin")
    _, result = apply(client, result["session"], "present")
    first = result["session"]["attempts"][0]
    assert first["supported"]
    clocks(client, NOW + elapsed)
    time = (NOW + elapsed).isoformat()
    _, result = apply(client, result["session"], "rate", rating="recalled", occurred_at=time)
    _, result = apply(client, result["session"], "present", occurred_at=time)
    expected_support = elapsed <= timedelta(minutes=5)
    assert result["session"]["attempts"][1]["supported"] is expected_support
    body, result = apply(client, result["session"], "rate", rating="recalled", occurred_at=time)
    assert result["session"]["attempts"][1]["original"]["supported"] is expected_support
    saved = client.get(f"/api/v1/sessions/{session['id']}").json()
    assert saved["attempts"][0]["original"]["supported"]
    replay = client.post(f"/api/v1/sessions/{session['id']}/events", json={"events": [body]})
    assert replay.json()["receipts"] == result["receipts"]
