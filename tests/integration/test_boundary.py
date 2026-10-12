import json
from phrase_gym.api.app import create_app
from tests.integration.helpers import ROOT


def test_cross_site_unknown_host_and_body_limit(client):
    settings = {"native_language": "ru"}
    assert (
        client.put(
            "/api/v1/settings", json=settings, headers={"Origin": "https://untrusted.example"}
        ).status_code
        == 403
    )
    assert (
        client.put(
            "/api/v1/settings", json=settings, headers={"Sec-Fetch-Site": "cross-site"}
        ).status_code
        == 403
    )
    assert client.get("/api/v1/settings", headers={"Host": "untrusted.example"}).status_code == 400
    assert (
        client.put(
            "/api/v1/settings", json=settings, headers={"Origin": "http://testserver"}
        ).status_code
        == 200
    )
    assert (
        client.put(
            "/api/v1/settings", content=b"{}", headers={"Content-Length": str(25 * 1024 * 1024)}
        ).status_code
        == 413
    )


def test_schema_snapshot_matches_api():
    assert create_app().openapi() == json.loads(
        (ROOT / "frontend/openapi.json").read_text(encoding="utf-8")
    )
