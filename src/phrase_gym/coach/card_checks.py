import re
from typing import Any

from pydantic import ValidationError

from phrase_gym.coach.hint_checks import (
    bold_expressions,
    contains_words,
    gloss_problems,
    hint_problems,
    hint_set_problems,
    words,
)
from phrase_gym.server.import_contracts import ImportPhrase

GAP = "__________"
GAP_RUN = re.compile(r"_{2,}")
CUE_COUNTS = {"phrase": (1, 2), "word": (2, 3)}


def phrase_of(card: dict[str, Any], item_id: str) -> ImportPhrase:
    cues = [
        {key: value for key, value in cue.items() if value is not None}
        for cue in card.get("cues") or []
        if isinstance(cue, dict)
    ]
    fields = {key: value for key, value in card.items() if value is not None}
    return ImportPhrase.model_validate({**fields, "cues": cues, "item_id": item_id})


def _validation_problems(error: ValidationError) -> list[str]:
    return [
        f"{'.'.join(str(part) for part in item['loc']) or 'card'}: {item['msg']}"
        for item in error.errors()
    ]


def _cue_problems(phrase: ImportPhrase, kind: str, target: list[str]) -> list[str]:
    problems = []
    low, high = CUE_COUNTS[kind]
    if not low <= len(phrase.cues) <= high:
        problems.append(f"cues: a {kind} card needs {low} to {high} cues")
    for index, cue in enumerate(phrase.cues):
        if GAP_RUN.findall(cue.situation) != [GAP]:
            problems.append(f"cues.{index}.situation: use exactly one {GAP} gap")
        visible = f"{cue.situation} {cue.partner_line or ''}"
        if target and contains_words(visible, target):
            problems.append(f"cues.{index}: the cue shows the complete target")
    return problems


def card_problems(answer: dict[str, Any], item_id: str) -> list[str]:
    """Structural checks; semantic quality stays with the reviewer."""
    card = answer.get("card")
    if not isinstance(card, dict):
        return ["card: missing"]
    kind = answer.get("kind")
    if kind not in CUE_COUNTS:
        return ["kind: must be phrase or word"]
    target = words(str(answer.get("target") or ""))
    if not target:
        return ["target: missing"]
    try:
        phrase = phrase_of(card, item_id)
    except ValidationError as error:
        return _validation_problems(error)
    problems = _cue_problems(phrase, kind, target)
    problems += [
        f"recall_hints.{index}: the hint gives the complete target"
        for index, hint in enumerate(phrase.recall_hints)
        if contains_words(hint, target)
    ]
    for index, hint in enumerate(phrase.recall_hints):
        problems += hint_problems(index, hint, str(answer["target"]), phrase.cues)
    contrasted = bold_expressions(phrase.explanation, phrase.detailed_explanation)
    problems += hint_set_problems(phrase.recall_hints, str(answer["target"]), contrasted)
    problems += gloss_problems(phrase.cues, str(answer["target"]), phrase.accepted_variants)
    problems += [
        f"recall_hints.{index}: the hint shows the accepted variant “{variant}”; "
        "keep the variant and use another anchor in the hint"
        for index, hint in enumerate(phrase.recall_hints)
        for variant in phrase.accepted_variants
        if contains_words(hint, words(variant))
    ]
    return problems
