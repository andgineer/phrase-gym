import json
import shutil
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Protocol

from phrase_gym.coach.card_checks import card_problems, phrase_of
from phrase_gym.coach.prompt_text import language_values, render, schema
from phrase_gym.server.coach_contracts import CoachCorrection, CoachWork
from phrase_gym.server.import_contracts import ImportPhrase


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


@dataclass(frozen=True)
class PreparedPhrase:
    phrase: ImportPhrase
    correction: CoachCorrection | None


def prepared(answer: dict[str, Any], item_id: str) -> PreparedPhrase:
    correction = answer.get("correction")
    return PreparedPhrase(
        phrase_of(answer["card"], item_id),
        CoachCorrection.model_validate(correction) if correction else None,
    )


async def prepare_card(work: CoachWork, cli: Cli, attempt: Path) -> PreparedPhrase:
    item_id = work.package_id
    language = work.learning_language
    values = {**language_values(work), "input": _shown(work.text)}
    card_schema = schema("card.schema.json")
    final = _ready(
        await cli.ask(attempt, "prepare", render("prepare.md", **values), card_schema),
    )
    for review_step, revise_step in REVISION_ROUNDS:
        problems = card_problems(final, item_id, work.text, language)
        review = await cli.ask(
            attempt,
            review_step,
            render("review.md", **values, draft=_shown(final)),
            schema("review.schema.json"),
        )
        findings = _findings(review)
        if not findings and not problems:
            break
        prompt = render(
            "revise.md",
            **values,
            draft=_shown(final),
            findings=_shown(findings),
            problems=_shown(problems),
        )
        final = _ready(await cli.ask(attempt, revise_step, prompt, card_schema))
    problems = card_problems(final, item_id, work.text, language)
    if problems:
        raise PreparationError("The prepared card failed its checks: " + "; ".join(problems))
    return prepared(final, item_id)


class CardPreparer:
    def __init__(self, cli: Cli, attempts: Path):
        self.cli = cli
        self.attempts = attempts

    async def prepare(self, work: CoachWork) -> PreparedPhrase:
        stamp = datetime.now(UTC).strftime("%Y%m%d-%H%M%S-%f")
        attempt = self.attempts / f"{stamp}-{work.package_id}"
        try:
            phrase = await prepare_card(work, self.cli, attempt)
        except PreparationError as error:
            raise PreparationError(str(error), attempt, retry_helps=error.retry_helps) from error
        except (RuntimeError, OSError) as error:
            raise PreparationError(str(error), attempt) from error
        # Failed attempts stay for diagnosis; a published card needs no private trail.
        shutil.rmtree(attempt, ignore_errors=True)
        return phrase
