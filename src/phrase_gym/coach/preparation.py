import json
import re
import shutil
from datetime import UTC, datetime
from importlib.resources import files
from pathlib import Path
from typing import Any, Protocol

from phrase_gym.coach.card_checks import card_problems, phrase_of
from phrase_gym.server.import_contracts import ImportPhrase

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
    shared = {"rules": _resource("content-rules.md"), "card_format": _resource("card-format.md")}
    fields = {**shared, **values}
    # One pass keeps placeholder-like text inside learner input or model output literal.
    return PLACEHOLDER.sub(lambda match: fields[match[1]].strip(), _resource(template))


def _shown(value: object) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2)


def _ready(answer: dict[str, Any]) -> dict[str, Any]:
    if answer.get("outcome") == "insufficient":
        reason = str(answer.get("reason") or "").strip()
        # The model judged the input itself unusable, so typing it again cannot help.
        raise PreparationError(
            reason or "Coach could not prepare a card from this input",
            retry_helps=False,
        )
    if answer.get("outcome") != "ready":
        raise PreparationError("The preparation CLI returned an answer without a ready card")
    return answer


def _findings(review: dict[str, Any]) -> list[dict[str, Any]]:
    findings = review.get("findings")
    if not isinstance(findings, list):
        raise PreparationError("The preparation CLI returned a review without findings")
    return [finding for finding in findings if isinstance(finding, dict)]


# A corrected card is reviewed once more; at most two reviews and two revisions per card.
REVISION_ROUNDS = (("review", "revise"), ("review-2", "revise-2"))


async def _review(cli: Cli, attempt: Path, step: str, typed: str, card: dict[str, Any]) -> list:
    review = await cli.ask(
        attempt,
        step,
        render("review.md", input=typed, draft=_shown(card)),
        schema("review.schema.json"),
    )
    return _findings(review)


async def prepare_card(text: str, item_id: str, cli: Cli, attempt: Path) -> ImportPhrase:
    typed = _shown(text)
    card_schema = schema("card.schema.json")
    final = _ready(
        await cli.ask(attempt, "prepare", render("prepare.md", input=typed), card_schema),
    )
    for review_step, revise_step in REVISION_ROUNDS:
        problems = card_problems(final, item_id)
        findings = await _review(cli, attempt, review_step, typed, final)
        if not findings and not problems:
            break
        prompt = render(
            "revise.md",
            input=typed,
            draft=_shown(final),
            findings=_shown(findings),
            problems=_shown(problems),
        )
        final = _ready(await cli.ask(attempt, revise_step, prompt, card_schema))
    problems = card_problems(final, item_id)
    if problems:
        raise PreparationError("The prepared card failed its checks: " + "; ".join(problems))
    return phrase_of(final["card"], item_id)


class CardPreparer:
    def __init__(self, cli: Cli, attempts: Path):
        self.cli = cli
        self.attempts = attempts

    async def prepare(self, text: str, item_id: str, learning_language: str) -> ImportPhrase:
        if learning_language != "en":
            raise PreparationError(
                "Coach prepares cards with examples for English only.",
                retry_helps=False,
            )
        stamp = datetime.now(UTC).strftime("%Y%m%d-%H%M%S-%f")
        attempt = self.attempts / f"{stamp}-{item_id}"
        try:
            phrase = await prepare_card(text, item_id, self.cli, attempt)
        except PreparationError as error:
            raise PreparationError(str(error), attempt, retry_helps=error.retry_helps) from error
        except (RuntimeError, OSError) as error:
            raise PreparationError(str(error), attempt) from error
        # Failed attempts stay for diagnosis; a published card needs no private trail.
        shutil.rmtree(attempt, ignore_errors=True)
        return phrase
