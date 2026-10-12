from fastapi.testclient import TestClient
from phrase_gym.api.app import create_app


def test_first_launch_settings_survive_restart(config):
    with TestClient(create_app(config)) as client:
        assert client.get("/api/v1/settings").json()["native_language"] is None
        assert (
            client.put(
                "/api/v1/settings", json={"native_language": "ru", "ui_language_mode": "learning"}
            ).status_code
            == 200
        )
    with TestClient(create_app(config)) as client:
        assert client.get("/api/v1/settings").json()["native_language"] == "ru"
        assert client.get("/api/v1/settings").json() == {
            "native_language": "ru",
            "ui_language_mode": "learning",
        }
        assert (
            client.patch(
                "/api/v1/settings/language", json={"last_learning_language": "sr"}
            ).status_code
            == 404
        )
        assert (
            client.put(
                "/api/v1/settings", json={"native_language": "en", "last_learning_language": "sr"}
            ).status_code
            == 422
        )
