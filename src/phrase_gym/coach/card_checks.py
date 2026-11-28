import re
from typing import Any

from pydantic import ValidationError

from phrase_gym.coach.companion_checks import (
    companion_problems,
    gloss_frame_problems,
    repeated_gloss_problems,
)
from phrase_gym.coach.hint_checks import (
    bold_expressions,
    contains_words,
    gloss_problems,
    hint_problems,
    hint_set_problems,
    words,
)
from phrase_gym.server.import_contracts import ImportPhrase
from phrase_gym.server.languages import LearningLanguage
from phrase_gym.server.sentence_gaps import GAP

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


def _whole_words(part: str, whole: str) -> bool:
    return part in whole and contains_words(whole, words(part))


def correction_problems(answer: dict[str, Any], typed: str) -> list[str]:
    correction = answer.get("correction")
    if correction is None:
        return []
    if not isinstance(correction, dict) or any(
        not isinstance(correction.get(key), str) or not correction[key].strip()
        for key in ("original", "corrected")
    ):
        return ["correction: give the original and corrected wording, or null"]
    original, corrected = correction["original"], correction["corrected"]
    target = str(answer.get("target") or "")
    checks = [
        (original == corrected, "correction: the corrected wording must differ from the original"),
        (
            not _whole_words(original, typed),
            "correction: the original must be whole words copied exactly from the input",
        ),
        (
            not _whole_words(corrected, target),
            "correction: the corrected wording must be the target or whole words of it",
        ),
    ]
    return [problem for failed, problem in checks if failed]


def card_problems(
    answer: dict[str, Any],
    item_id: str,
    typed: str,
    language: LearningLanguage,
) -> list[str]:
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
    problems = correction_problems(answer, typed) + _cue_problems(phrase, kind, target)
    problems += [
        f"recall_hints.{index}: the hint gives the complete target"
        for index, hint in enumerate(phrase.recall_hints)
        if contains_words(hint, target)
    ]
    target_text = str(answer["target"])
    for index, hint in enumerate(phrase.recall_hints):
        problems += hint_problems(index, hint, target_text, phrase.cues, language)
    contrasted = bold_expressions(phrase.explanation, phrase.detailed_explanation)
    problems += hint_set_problems(phrase.recall_hints, target_text, contrasted, language)
    problems += companion_problems(
        phrase.recall_hints,
        target_text,
        phrase.cues,
        phrase.text,
        language,
    )
    problems += gloss_frame_problems(phrase.cues)
    problems += repeated_gloss_problems(phrase.cues)
    problems += gloss_problems(phrase.cues, target_text)
    return problems
