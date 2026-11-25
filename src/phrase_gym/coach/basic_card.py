import re

from phrase_gym.coach.hint_checks import contains_words, words
from phrase_gym.server.coach_contracts import GAP, BasicCard, CoachCorrection

INSTRUCTIONS = {"en": "Say the missing wording."}
WORD_CHAR = r"[\w'’\-]"
LETTER_RUN = re.compile(r"[^\W\d_]+")
LONG_WORD = 7


def contains_target(text: str, target: str) -> bool:
    return bool(words(target) and contains_words(text, words(target)))


def gapped(source: str | None, target: str) -> str | None:
    """The source with its single exact target span replaced by a gap, if that hides the answer."""
    if not source or GAP in source:
        return None
    pattern = re.compile(rf"(?<!{WORD_CHAR}){re.escape(target)}(?!{WORD_CHAR})")
    matches = list(pattern.finditer(source))
    if len(matches) != 1:
        return None
    before, after = matches[0].span()
    masked = source[:before] + GAP + source[after:]
    return None if contains_target(masked, target) else masked


def partial_form(target: str) -> str | None:
    """A partial form of the target's longest word, such as def…ly for definitely."""
    runs = [run.group() for run in LETTER_RUN.finditer(target)]
    if not runs:
        return None
    token = max(runs, key=len)
    if len(token) < 2:  # noqa: PLR2004
        return None
    head = token[: min(3, len(token) // 3 or 1)]
    tail = token[-2:] if len(token) >= LONG_WORD else ""
    partial = f"{head}…{tail}"
    return None if contains_target(partial, target) else partial


def basic_card(
    target: str,
    original: str,
    source: str | None,
    card_language: str = "en",
) -> BasicCard | None:
    """Wording practice from the confirmed target; None when no cue can conceal the answer."""
    instruction = INSTRUCTIONS[card_language]
    masked = gapped(source, target)
    cue = masked or partial_form(target)
    if not cue or contains_target(instruction, target):
        return None
    correction = (
        CoachCorrection(original=original, corrected=target) if original != target else None
    )
    return BasicCard(
        target=target,
        text=source if masked and source else target,
        cue=cue,
        instruction=instruction,
        correction=correction,
    )
