from datetime import timedelta

from sqlalchemy import select

from phrase_gym.storage import Database
from phrase_gym.tables import ScheduleRow, SessionRow
from tests.integration.helpers import NOW, publish
from tests.integration.session_helpers import ready_package


def test_readiness_is_scoped_read_only_and_keeps_future_due_utc(client, package, config):
    content = ready_package(client, package)
    database = Database(config.data_dir)
    try:
        with database.write() as transaction:
            transaction.add(
                ScheduleRow(
                    target_id=content[0]["id"],
                    version=1,
                    due_at=(NOW + timedelta(days=10)).isoformat(),
                )
            )
            transaction.add(
                ScheduleRow(target_id=content[1]["id"], version=4, due_at=NOW.isoformat())
            )
        response = client.get("/api/v1/sessions/readiness?language=en")
        assert response.status_code == 200, response.text
        result = response.json()
        assert result["due_count"] == 1 and result["new_count"] == 1
        assert result["preparing_count"] == 0
        assert result["next_due_at"] == (NOW + timedelta(days=10)).isoformat().replace(
            "+00:00", "Z"
        )
        assert client.get("/api/v1/sessions/readiness?language=sr").json()["due_count"] == 0
        with database.read() as transaction:
            assert transaction.scalars(select(SessionRow)).all() == []
            assert len(transaction.scalars(select(ScheduleRow)).all()) == 2
    finally:
        database.close()


def test_unprepared_targets_are_not_presented_as_ready_due_work(client, package):
    publish(client, package)
    result = client.get("/api/v1/sessions/readiness?language=en").json()
    assert result["preparing_count"] == 3
    assert result["new_count"] == 0 and result["next_due_at"] is None
