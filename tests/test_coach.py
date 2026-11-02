import pytest
from fastapi.testclient import TestClient

from phrase_gym.coach.main import CoachApi, process_once
from phrase_gym.coach.speech import PiperSpeech
from tests.coach_client import ClientCoachApi
from tests.phrase_packages import Package


@pytest.mark.parametrize(
    "url",
    ["http://127.0.0.1:8000", "http://localhost:8941/"],
)
def test_coach_accepts_only_canonical_loopback_authority(url: str) -> None:
    assert CoachApi(url).base_url == url.rstrip("/")


@pytest.mark.parametrize(
    "url",
    [
        "http://127.0.0.1:8000@attacker.example",
        "http://user@localhost:8000",
        "https://localhost:8000",
        "http://attacker.example:8000",
        "http://localhost",
        "http://localhost:0",
        "http://localhost:99999",
        "http://localhost:8000/other",
        "http://localhost:8000?next=attacker.example",
    ],
)
def test_coach_rejects_non_loopback_or_ambiguous_url(url: str) -> None:
    with pytest.raises(ValueError, match="loopback"):
        CoachApi(url)


def test_coach_prints_a_failed_preparation(
    client: TestClient, package: Package, capsys: pytest.CaptureFixture[str]
) -> None:
    client.post("/api/imports", json={"package": package})
    assert process_once(ClientCoachApi(client), PiperSpeech(None, fake=True, fake_failures=1))
    assert capsys.readouterr().out == (
        f"Coach could not prepare {package['package_id']}: "
        "Synthetic preparation failure; correct the setup and retry\n"
    )
    status = client.get(f"/api/imports/{package['package_id']}").json()
    assert status["error"] == "Synthetic preparation failure; correct the setup and retry"
