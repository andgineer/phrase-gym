import mimetypes
import shutil
import struct
from pathlib import Path
from xml.etree import ElementTree

import pytest
from fastapi.testclient import TestClient

from phrase_gym.server import app

FRONTEND = Path(__file__).parents[1] / "frontend"


@pytest.fixture
def installed_client(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    # Vite copies public assets beside index.html in the packaged static directory.
    static = tmp_path / "static"
    shutil.copytree(FRONTEND / "public", static)
    shutil.copyfile(FRONTEND / "index.html", static / "index.html")
    monkeypatch.setattr(app, "__file__", str(tmp_path / "app.py"))
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path / "data"))
    mimetypes.add_type("text/plain", ".webmanifest")
    with TestClient(app.create_app()) as client:
        yield client


def test_packaged_manifest_and_icons_are_served(installed_client: TestClient):
    response = installed_client.get("/manifest.webmanifest")
    assert response.headers["content-type"] == "application/manifest+json"
    manifest = response.json()
    assert manifest == {
        "name": "Phrase Gym",
        "short_name": "Phrase Gym",
        "start_url": "/",
        "display": "standalone",
        "background_color": "#f7f4ec",
        "theme_color": "#f7f4ec",
        "icons": [
            {"src": "/icon-192.png", "sizes": "192x192", "type": "image/png", "purpose": "any"},
            {
                "src": "/icon-512.png",
                "sizes": "512x512",
                "type": "image/png",
                "purpose": "maskable",
            },
        ],
        "shortcuts": [{"name": "Coach", "url": "/#coach"}],
    }
    for size in (180, 192, 512):
        icon = installed_client.get(f"/icon-{size}.png")
        assert icon.headers["content-type"] == "image/png"
        assert icon.content[:8] == b"\x89PNG\r\n\x1a\n"
        assert struct.unpack(">II", icon.content[16:24]) == (size, size)
    favicon = installed_client.get("/icon.svg")
    assert favicon.headers["content-type"] == "image/svg+xml"
    assert ElementTree.fromstring(favicon.content).tag == "{http://www.w3.org/2000/svg}svg"


def test_packaged_index_links_installation_assets(installed_client: TestClient):
    html = installed_client.get("/").text
    for snippet in (
        'name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover"',
        'name="theme-color" content="#f7f4ec"',
        'name="apple-mobile-web-app-capable" content="yes"',
        'name="apple-mobile-web-app-title" content="Phrase Gym"',
        'rel="manifest" href="/manifest.webmanifest"',
        'rel="apple-touch-icon" sizes="180x180" href="/icon-180.png"',
        'rel="icon" href="/icon.svg"',
    ):
        assert snippet in html
