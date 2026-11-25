from dataclasses import replace
from pathlib import Path

from phrase_gym.coach.agents.config import CLAUDE_CHECKER, CODEX_CHECKER
from phrase_gym.coach.checked_preparation import CheckedPreparer
from phrase_gym.coach.claude_cli import ClaudeCli
from phrase_gym.coach.codex_cli import CodexCli
from phrase_gym.coach.gemini_cli import GeminiCli
from phrase_gym.coach.recognition import Recognizer
from tests import fake_cli

SubscriptionCli = ClaudeCli | CodexCli


def subscription_writer(vendor: str, directory: Path, timeout: float = 10) -> SubscriptionCli:
    fake = str(fake_cli.executable(directory, vendor))
    if vendor == "claude":
        return ClaudeCli(fake, timeout=timeout, attempts=directory / "attempts")
    workspace = directory / "workspace"
    workspace.mkdir(exist_ok=True)
    return CodexCli(workspace, fake, timeout=timeout)


def checked_preparer(vendor: str, directory: Path, timeout: float = 10) -> CheckedPreparer:
    writer = subscription_writer(vendor, directory, timeout)
    checker = replace(writer, model=CLAUDE_CHECKER if vendor == "claude" else CODEX_CHECKER)
    return CheckedPreparer(writer, checker, writer.attempts, directory / "usage")


def free_recognizer(directory: Path, timeout: float = 10) -> Recognizer:
    fake = str(fake_cli.executable(directory, "gemini"))
    gemini = GeminiCli(fake, timeout=timeout, attempts=directory / "attempts", launch_delay=0)
    return Recognizer(gemini, gemini.attempts, directory / "usage")
