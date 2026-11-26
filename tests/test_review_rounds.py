import asyncio
from pathlib import Path

import pytest

from phrase_gym.coach.claude_cli import ClaudeCli
from phrase_gym.coach.codex_cli import CodexCli
from phrase_gym.coach.preparation import PreparationError, prepare_card
from phrase_gym.server.coach_contracts import CoachWork
from tests import fake_cli

Cli = ClaudeCli | CodexCli


@pytest.fixture(params=["claude", "codex"])
def cli(request: pytest.FixtureRequest, tmp_path: Path) -> Cli:
    fake = str(fake_cli.executable(tmp_path, request.param))
    if request.param == "claude":
        return ClaudeCli(fake, timeout=10, attempts=tmp_path / "attempts")
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    return CodexCli(workspace, fake, timeout=10)


def typed(text: str) -> CoachWork:
    return CoachWork(
        package_id="coach-1",
        text=text,
        learning_language="en",
        native_language="ru",
        translation=False,
    )


def calls(attempt: Path) -> list[str]:
    order = ["prepare", "review", "revise", "review-2", "revise-2"]
    made = {path.name.removesuffix("-prompt.md") for path in attempt.glob("*-prompt.md")}
    return [step for step in order if step in made]


@pytest.mark.parametrize(
    ("text", "steps"),
    [
        ("clean", ["prepare", "review"]),
        ("leaky", ["prepare", "review", "revise", "review-2"]),
        ("picky", ["prepare", "review", "revise", "review-2", "revise-2"]),
        ("sticky", ["prepare", "review", "revise", "review-2", "revise-2"]),
    ],
    ids=[
        "clean first review",
        "clean second review",
        "two revisions",
        "first revision fails its checks",
    ],
)
def test_a_corrected_card_is_reviewed_again_before_publication(
    cli: Cli, tmp_path: Path, text: str, steps: list[str]
) -> None:
    attempt = tmp_path / "attempt"
    phrase = asyncio.run(prepare_card(typed(text), cli, attempt))
    assert phrase.text == f"She felt {text} today."
    assert calls(attempt) == steps


def test_a_second_revision_that_fails_its_checks_is_not_published(cli: Cli, tmp_path: Path) -> None:
    attempt = tmp_path / "attempt"
    with pytest.raises(PreparationError, match="failed its checks: recall_hints.0"):
        asyncio.run(prepare_card(typed("stubborn"), cli, attempt))
    assert calls(attempt) == ["prepare", "review", "revise", "review-2", "revise-2"]
    assert len(list(attempt.glob("*-prompt.md"))) == 5


def test_the_second_review_judges_the_corrected_card_without_earlier_findings(
    cli: Cli, tmp_path: Path
) -> None:
    attempt = tmp_path / "attempt"
    asyncio.run(prepare_card(typed("leaky"), cli, attempt))
    second = (attempt / "review-2-prompt.md").read_text()
    assert "The hint gives the whole target." not in second
    assert '"le... / one"' in second


def test_a_revision_that_finds_the_input_insufficient_ends_without_a_retry(
    cli: Cli, tmp_path: Path
) -> None:
    attempt = tmp_path / "attempt"
    with pytest.raises(PreparationError) as raised:
        asyncio.run(prepare_card(typed("giveup"), cli, attempt))
    assert raised.value.retry_helps is False
    assert str(raised.value) == "No English word or expression was found in the input."
    assert calls(attempt) == ["prepare", "review", "revise"]
