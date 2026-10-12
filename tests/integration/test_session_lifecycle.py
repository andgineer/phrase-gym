from concurrent.futures import ThreadPoolExecutor
from uuid import UUID, uuid4

import pytest
from fastapi.testclient import TestClient
from sqlalchemy import select
from sqlalchemy.exc import IntegrityError

from phrase_gym.api.app import create_app
from phrase_gym.contracts.sessions import SessionStart
from phrase_gym.errors import DomainError
from phrase_gym.storage import Database, SessionRow
from tests.integration.helpers import NOW
from tests.integration.session_helpers import ready_package, start


def test_start_idempotency_and_payload_conflict_survive_restart(client, package, config):
    ready_package(client, package)
    payload = {"id": str(uuid4()), "language": "en"}
    first = client.post("/api/v1/sessions", json=payload)
    assert first.status_code == 200
    assert client.post("/api/v1/sessions", json=payload).json() == first.json()
    assert client.post("/api/v1/sessions", json={**payload, "language": "sr"}).status_code == 409
    with TestClient(create_app(config, clock=lambda: NOW)) as restarted:
        assert restarted.post("/api/v1/sessions", json=payload).json() == first.json()
        assert restarted.get("/api/v1/sessions/current").json() == first.json()


def test_pause_resume_retry_and_stale_transition(client, package):
    ready_package(client, package)
    first = start(client).json()
    path = f"/api/v1/sessions/{first['id']}"
    pause = {"expected_version": 1, "status": "paused"}
    paused = client.patch(path, json=pause).json()
    assert paused["status"] == "paused" and paused["version"] == 2
    assert paused["manifest"] == first["manifest"]
    assert client.patch(path, json=pause).json() == paused
    assert start(client).status_code == 409  # Pause retains the single open slot.
    assert client.patch(path, json={"expected_version": 1, "status": "active"}).status_code == 409
    resumed = client.patch(path, json={"expected_version": 2, "status": "active"}).json()
    assert resumed["version"] == 3
    assert resumed["manifest"] == first["manifest"]
    assert client.patch(path, json=pause).status_code == 409
    assert client.get(f"/api/v1/sessions/{uuid4()}").status_code == 404
    assert client.patch(f"/api/v1/sessions/{uuid4()}", json=pause).status_code == 404


def test_explicit_takeover_supersedes_without_reusing_manifest_or_groups(client, package):
    ready_package(client, package)
    first = start(client).json()
    assert start(client).status_code == 409
    assert start(client, takeover_session_id=str(uuid4())).status_code == 409
    second = start(client, takeover_session_id=first["id"]).json()
    old_path = f"/api/v1/sessions/{first['id']}"
    old = client.get(old_path).json()
    assert old["status"] == "superseded" and old["version"] == 2
    assert old["manifest"] == first["manifest"]
    assert (
        client.patch(old_path, json={"expected_version": 2, "status": "active"}).status_code == 409
    )
    assert start(client, takeover_session_id=first["id"]).status_code == 409
    assert client.get("/api/v1/sessions/current").json() == second
    assert {t["review_group_id"] for t in first["manifest"]["targets"]}.isdisjoint(
        t["review_group_id"] for t in second["manifest"]["targets"]
    )


def test_simultaneous_starts_serialize_and_database_enforces_one_open_slot(client, package, config):
    ready_package(client, package)
    service = client.app.state.sessions

    def attempt(_):
        try:
            return service.start(SessionStart(id=uuid4(), language="en")).id
        except DomainError as error:
            assert error.status == 409
            return None

    with ThreadPoolExecutor(max_workers=2) as workers:
        results = list(workers.map(attempt, range(2)))
    assert sum(isinstance(result, UUID) for result in results) == 1
    database = Database(config.data_dir)
    try:
        with pytest.raises(IntegrityError), database.write() as transaction:
            row = transaction.scalar(select(SessionRow))
            transaction.add(
                SessionRow(
                    id=str(uuid4()),
                    start_payload=row.start_payload,
                    manifest=row.manifest,
                    status="paused",
                    open_slot=1,
                    version=1,
                    created_at=row.created_at,
                    updated_at=row.updated_at,
                )
            )
    finally:
        database.close()
