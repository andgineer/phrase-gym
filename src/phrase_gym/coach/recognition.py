import json
import shutil
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from phrase_gym.coach.basic_card import confirmed_card
from phrase_gym.coach.messages import LANGUAGE_NAMES
from phrase_gym.coach.preparation import Cli, PreparationError, render, schema
from phrase_gym.server.coach_contracts import BasicCard, CoachWork

FIELDS = {
    "outcome": str,
    "original_span": str,
    "target": str,
    "source": str,
    "source_span": str,
    "source_usable": bool,
    "reason": str,
}


def recognition_problem(answer: dict[str, Any], text: str) -> str | None:
    if any(not isinstance(answer.get(name), kind) for name, kind in FIELDS.items()):
        return "The recognition answer is missing required fields"
    if answer["outcome"] == "reject":
        return None if answer["reason"].strip() else "The rejection gives no reason"
    original, target, source = answer["original_span"], answer["target"], answer["source"]
    checks = [
        (answer["outcome"] != "accept", "The recognition answer has no outcome"),
        (
            not original or not target.strip() or target != target.strip(),
            "The acceptance lacks the original span or the confirmed target",
        ),
        (original not in text, "The original span cannot be traced to the submitted text"),
        (
            bool(source) and source not in text,
            "The source sentence cannot be traced to the submitted text",
        ),
        (
            answer["source_usable"] and not source,
            "The answer confirms a source that was not supplied",
        ),
    ]
    return next((problem for failed, problem in checks if failed), None)


def recognized_card(answer: dict[str, Any], work: CoachWork) -> BasicCard:
    if problem := recognition_problem(answer, work.text):
        raise PreparationError(problem)
    if answer["outcome"] == "reject":
        # A language decision about the input: typing it again cannot change it.
        raise PreparationError(answer["reason"].strip(), retry_helps=False)
    usable = answer["source_usable"]
    return confirmed_card(
        answer["target"],
        answer["original_span"],
        answer["source"] if usable else None,
        answer["source_span"] if usable else "",
        work.native_language,
    )


def learner_input(work: CoachWork) -> str:
    typed = {
        "text": work.text,
        "learning_language": LANGUAGE_NAMES[work.learning_language],
        "native_language": LANGUAGE_NAMES[work.native_language],
    }
    return json.dumps(typed, ensure_ascii=False)


class Recognizer:
    """One recognition call confirms the target; the program builds basic wording practice."""

    def __init__(self, cli: Cli, attempts: Path):
        self.cli = cli
        self.attempts = attempts

    async def prepare(self, work: CoachWork) -> BasicCard:
        stamp = datetime.now(UTC).strftime("%Y%m%d-%H%M%S-%f")
        attempt = self.attempts / f"{stamp}-{work.package_id}"
        prompt = render("recognize.md", input=learner_input(work))
        try:
            answer = await self.cli.ask(
                attempt,
                "recognize",
                prompt,
                schema("recognize.schema.json"),
            )
            card = recognized_card(answer, work)
        except PreparationError as error:
            raise PreparationError(str(error), attempt, retry_helps=error.retry_helps) from error
        except (RuntimeError, OSError) as error:
            raise PreparationError(str(error), attempt) from error
        # Failed attempts stay for diagnosis; a published card needs no private trail.
        shutil.rmtree(attempt, ignore_errors=True)
        return card
