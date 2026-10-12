from uuid import uuid4
from sqlalchemy import func, select
from phrase_gym.storage import Database
from phrase_gym.tables import ExposureRow, Target
from tests.integration.helpers import NOW, publish, phrases


def test_exposure_idempotent_clock_and_no_review(client, package, config):
    publish(client, package)
    phrase = phrases(client)[0]
    event = {
        "id": str(uuid4()),
        "target_id": phrase["id"],
        "revision": phrase["revision"],
        "language": "en",
        "kind": "explanation",
    }
    first = client.post("/api/v1/exposures", json=event)
    assert first.json()["received_at"] == NOW.isoformat()
    assert client.post("/api/v1/exposures", json=event).json() == first.json()
    event["kind"] = "answer"
    assert client.post("/api/v1/exposures", json=event).status_code == 409
    database = Database(config.data_dir)
    with database.read() as session:
        assert session.scalar(select(func.count()).select_from(ExposureRow)) == 1
        assert session.scalar(select(func.count()).select_from(Target)) == 3
    database.close()
