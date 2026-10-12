from uuid import uuid4

from tests.integration.helpers import NOW
from tests.integration.session_helpers import ready_package, start


def session_fixture(client, package, **kwargs):
    ready_package(client, package)
    response = start(client, **kwargs)
    assert response.status_code == 200, response.text
    return response.json()


def event(client, session, kind, **parameters):
    body = {
        "id": str(uuid4()),
        "sequence": session["last_sequence"] + 1,
        "step": session["cursor"],
        "kind": kind,
        "occurred_at": NOW.isoformat(),
        **parameters,
    }
    return body, client.post(f"/api/v1/sessions/{session['id']}/events", json={"events": [body]})


def apply(client, session, kind, **parameters):
    body, response = event(client, session, kind, **parameters)
    assert response.status_code == 200, response.text
    return body, response.json()
