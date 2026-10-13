from dataclasses import replace

from fastapi.testclient import TestClient

from phrase_gym.api.app import create_app


def test_online_client_serves_network_only_retirement_and_no_offline_assets(config, tmp_path):
    client_dir = tmp_path / "client"
    client_dir.mkdir()
    (client_dir / "index.html").write_text("<main>Online client</main>")
    (client_dir / "retire-worker.js").write_text("self.registration.unregister()")
    (client_dir / "icon.svg").write_text("<svg></svg>")
    (client_dir / "assets").mkdir()
    (client_dir / "assets/app.js").write_text("onlineClient()")
    with TestClient(create_app(replace(config, client_dir=client_dir))) as client:
        for path in ["/", "/index.html", "/learn/en/practice/retained"]:
            response = client.get(path)
            assert response.status_code == 200
            assert "Online client" in response.text
            assert response.headers["cache-control"] == "no-store"
        assert client.get("/sw.js").text == "self.registration.unregister()"
        for path in ["/sw.js", "/assets/app.js", "/icon.svg", "/api/v1/settings"]:
            assert client.get(path).headers["cache-control"] == "no-store"
        for path in ["/manifest.webmanifest", "/icon-192.png", "/apple-touch-icon.png"]:
            assert client.get(path).status_code == 404
