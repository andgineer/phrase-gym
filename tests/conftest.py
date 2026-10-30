from collections.abc import Iterator
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from phrase_gym.server.app import create_app
from tests.phrase_packages import Package, make_package


@pytest.fixture
def client(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Iterator[TestClient]:
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    monkeypatch.setenv("PHRASE_GYM_FAKE_TTS", "1")
    monkeypatch.delenv("PHRASE_GYM_FAKE_TTS_FAILURES", raising=False)
    with TestClient(create_app()) as running:
        yield running


@pytest.fixture
def package() -> Package:
    return make_package()
