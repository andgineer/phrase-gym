import pytest

from phrase_gym.coach.main import CoachApi


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
