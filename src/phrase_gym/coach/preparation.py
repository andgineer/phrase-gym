import json
import re
from importlib.resources import files
from pathlib import Path
from typing import Any, Protocol

from phrase_gym.coach.messages import LANGUAGE_NAMES
from phrase_gym.server.coach_contracts import CoachWork

PROMPTS = files("phrase_gym.coach").joinpath("prompts")
PLACEHOLDER = re.compile(r"\{\{(\w+)\}\}")


class PreparationError(RuntimeError):
    def __init__(self, reason: str, attempt: Path | None = None, *, retry_helps: bool = True):
        super().__init__(reason)
        self.attempt = attempt
        self.retry_helps = retry_helps


class Cli(Protocol):
    async def ask(
        self,
        attempt: Path,
        step: str,
        prompt: str,
        schema: dict[str, Any],
    ) -> dict[str, Any]: ...


def _resource(name: str) -> str:
    return PROMPTS.joinpath(name).read_text(encoding="utf-8")


def schema(name: str) -> dict[str, Any]:
    return json.loads(_resource(name))


def render(template: str, **values: str) -> str:
    shared = {
        "rules": _resource("content-rules.md"),
        "recognition": _resource("recognition-rules.md"),
    }
    fields = {**shared, **values}
    # One pass keeps placeholder-like text inside learner input or model output literal.
    return PLACEHOLDER.sub(lambda match: fields[match[1]].strip(), _resource(template))


def learner_input(work: CoachWork) -> str:
    typed = {
        "text": work.text,
        "learning_language": LANGUAGE_NAMES[work.learning_language],
        "card_language": LANGUAGE_NAMES[work.card_language],
        "help_language": LANGUAGE_NAMES[work.help_language],
    }
    return json.dumps(typed, ensure_ascii=False)
