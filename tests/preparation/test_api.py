from uuid import uuid4

from fastapi.testclient import TestClient

from phrase_gym.api.app import create_app
from phrase_gym.config import Config


def test_durable_source_and_handoff_api_without_any_provider_call(tmp_path, source):
    config = Config(tmp_path / "data")
    envelope = {
        "export_id": str(uuid4()),
        "sender": "echo-words",
        "selected_sense_id": "greeting",
        "selected_sense": "Greeting",
        "package": source.model_dump(mode="json"),
    }
    with TestClient(create_app(config)) as client:
        received = client.post("/api/v1/handoffs", json=envelope)
        assert received.status_code == 200 and received.json()["durable"]
        job_id = received.json()["preparation_id"]
    with TestClient(create_app(config)) as restarted:
        assert restarted.post("/api/v1/handoffs", json=envelope).json() == received.json()
        assert restarted.get(f"/api/v1/preparation/{job_id}").json()["status"] == "queued"
        envelope["selected_sense"] = "Changed sense"
        assert restarted.post("/api/v1/handoffs", json=envelope).status_code == 409
