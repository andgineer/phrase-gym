from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, ValidationError

from phrase_gym.coach.preparation import PreparationError
from phrase_gym.coach.recognition import recognition_problem


class Answer(BaseModel):
    # Schema-shaped CLI output: an unknown field or a coerced value means a malformed answer.
    model_config = ConfigDict(extra="forbid", strict=True)


class Recognition(Answer):
    outcome: Literal["accept", "reject"]
    original_span: str
    target: str
    source: str
    source_usable: bool
    kind: Literal["word", "phrase"]
    reason: str


class Example(Answer):
    text: str
    gloss: str
    partner_line: str
    attribution: Literal["source", "generated"]


class Judged(Answer):
    recognition: Recognition


class Draft(Judged):
    explanation: str
    examples: list[Example]
    recall_hints: list[str]
    usage_contexts: list[str]


class Check(Judged):
    explanation_ok: bool
    example_ok: list[bool]
    hints_ok: list[bool]
    usage_contexts_ok: list[bool]
    reasons: list[str]


def _validated[T: Judged](model: type[T], answer: dict[str, Any], text: str, step: str) -> T:
    try:
        parsed = model.model_validate(answer)
    except ValidationError as error:
        first = error.errors()[0]
        place = ".".join(str(part) for part in first["loc"])
        raise PreparationError(f"The {step} answer is malformed: {place}: {first['msg']}") from None
    if problem := recognition_problem(parsed.recognition.model_dump(), text):
        raise PreparationError(f"{problem} (during {step})")
    return parsed


def parse_draft(answer: dict[str, Any], text: str) -> Draft:
    return _validated(Draft, answer, text, "write")


def parse_check(answer: dict[str, Any], draft: Draft, text: str) -> Check:
    check = _validated(Check, answer, text, "check")
    counts = {
        "example_ok": (len(check.example_ok), len(draft.examples)),
        "hints_ok": (len(check.hints_ok), len(draft.recall_hints)),
        "usage_contexts_ok": (len(check.usage_contexts_ok), len(draft.usage_contexts)),
    }
    for name, (flags, items) in counts.items():
        if flags != items:
            raise PreparationError(
                f"The check answer is malformed: {name} judges {flags} of {items} items",
            )
    return check
