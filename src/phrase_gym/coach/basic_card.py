import re

from phrase_gym.coach.hint_checks import contains_words, words
from phrase_gym.server.coach_contracts import GAP, BasicCard, CoachCorrection

INSTRUCTIONS = {"en": "Say the missing wording."}
WORD_CHAR = r"[\w'’\-]"
TARGET_WORD = re.compile(r"[^\W\d_]+(?:['’-][^\W\d_]+)*")
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


def _fragment(word: str) -> str:
    # Every word keeps a slot; a one-letter word shows nothing rather than the whole word.
    if len(word) < 2:  # noqa: PLR2004
        return "…"
    head = word[: min(3, len(word) // 3 or 1)]
    tail = word[-2:] if len(word) >= LONG_WORD else ""
    return f"{head}…{tail}"


def partial_form(target: str) -> str | None:
    """Each word of the target shortened in order, such as st…ly be…ve for strongly believe."""
    fragments = [_fragment(word.group()) for word in TARGET_WORD.finditer(target)]
    if not any(fragment != "…" for fragment in fragments):
        return None
    partial = " ".join(fragments)
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
