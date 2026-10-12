import json
import pytest
from fastapi.testclient import TestClient
from phrase_gym.api.app import create_app
from phrase_gym.config import Config
from tests.integration.helpers import ROOT, NOW


@pytest.fixture
def package():
    return json.loads((ROOT / "fixtures/manual-en.json").read_text(encoding="utf-8"))


@pytest.fixture
def config(tmp_path):
    return Config(tmp_path / "data")


@pytest.fixture
def client(config):
    with TestClient(create_app(config, clock=lambda: NOW)) as client:
        yield client
