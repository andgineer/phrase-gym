import asyncio
from pathlib import Path

import pytest

from phrase_gym.coach.claude_cli import ClaudeCli
from phrase_gym.coach.codex_cli import CodexCli
from phrase_gym.coach.preparation import PreparationError, prepare_card
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


def calls(attempt: Path) -> list[str]:
    order = ["prepare", "review", "revise", "review-2", "revise-2"]
    made = {path.name.removesuffix("-prompt.md") for path in attempt.glob("*-prompt.md")}
    return [step for step in order if step in made]


@pytest.mark.parametrize(
    ("typed", "steps"),
    [
        ("clean", ["prepare", "review"]),
        ("leaky", ["prepare", "review", "revise", "review-2"]),
        ("picky", ["prepare", "review", "revise", "review-2", "revise-2"]),
    ],
    ids=["clean first review", "clean second review", "two revisions"],
)
def test_a_corrected_card_is_reviewed_again_before_publication(
    cli: Cli, tmp_path: Path, typed: str, steps: list[str]
) -> None:
    attempt = tmp_path / "attempt"
    phrase = asyncio.run(prepare_card(typed, "coach-1", cli, attempt))
    assert phrase.text == f"She felt {typed} today."
    assert calls(attempt) == steps


def test_a_second_revision_that_fails_its_checks_is_not_published(cli: Cli, tmp_path: Path) -> None:
    attempt = tmp_path / "attempt"
    with pytest.raises(PreparationError, match="failed its checks: recall_hints.0"):
        asyncio.run(prepare_card("stubborn", "coach-1", cli, attempt))
    assert calls(attempt) == ["prepare", "review", "revise", "review-2", "revise-2"]
    assert len(list(attempt.glob("*-prompt.md"))) == 5


def test_the_second_review_judges_the_corrected_card_without_earlier_findings(
    cli: Cli, tmp_path: Path
) -> None:
    attempt = tmp_path / "attempt"
    asyncio.run(prepare_card("leaky", "coach-1", cli, attempt))
    second = (attempt / "review-2-prompt.md").read_text()
    assert "The hint gives the whole target." not in second
    assert '"le... / one"' in second
