import pytest
from fastapi.testclient import TestClient

from phrase_gym.server.app import create_app
from phrase_gym.server.boundary import tailscale_host


@pytest.mark.parametrize(
    "origin",
    [
        "http://macbook.example.ts.net",
        "https://example.com",
        "https://macbook.example.ts.net/path",
        "https://macbook.example.ts.net:444",
        "https://user@macbook.example.ts.net",
        "https://macbook.example.ts.net?query=1",
        "https://macbook.example.ts.net#fragment",
    ],
)
def test_remote_origin_requires_one_https_tailscale_host(origin):
    with pytest.raises(ValueError, match="Tailscale origin"):
        tailscale_host(origin)


def test_tailscale_access_is_opt_in_and_same_origin(tmp_path, monkeypatch):
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    monkeypatch.delenv("PHRASE_GYM_TAILSCALE_ORIGIN", raising=False)
    host = "macbook.example.ts.net"
    with TestClient(create_app()) as local:
        assert local.get("/api/health", headers={"host": host}).status_code == 400

    monkeypatch.setenv("PHRASE_GYM_TAILSCALE_ORIGIN", f"https://{host}")
    with TestClient(create_app()) as client:
        assert client.get("/api/health", headers={"host": host}).status_code == 200
        assert (
            client.get("/api/health", headers={"host": "other.example.ts.net"}).status_code == 400
        )
        for origin in (f"http://{host}", "https://attacker.example", "null"):
            assert (
                client.post(
                    "/api/study/current", headers={"host": host, "origin": origin}
                ).status_code
                == 403
            )
        assert (
            client.post(
                "/api/study/current",
                headers={
                    "host": host,
                    "origin": f"https://{host}",
                    "sec-fetch-site": "same-origin",
                },
            ).status_code
            == 200
        )
        assert (
            client.post(
                "/api/study/current",
                headers={"host": host, "origin": f"https://{host}", "sec-fetch-site": "cross-site"},
            ).status_code
            == 403
        )
