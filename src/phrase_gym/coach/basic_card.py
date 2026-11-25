import re

from pydantic import ValidationError

from phrase_gym.coach.hint_checks import contains_words, words
from phrase_gym.coach.messages import message
from phrase_gym.coach.preparation import PreparationError
from phrase_gym.server.coach_contracts import BasicCard, CoachCorrection
from phrase_gym.server.sentence_gaps import GAP, realizes, shows_target

WORD_CHAR = r"[\w'’\-]"
TARGET_WORD = re.compile(r"[^\W\d_]+(?:['’-][^\W\d_]+)*")
TAIL_LETTER_WORD = 4
LONG_WORD = 7


def contains_target(text: str, target: str) -> bool:
    return bool(words(target) and contains_words(text, words(target)))


def _gap(source: str, span: str, target: str) -> str | None:
    if not realizes(span, target):
        return None
    pattern = re.compile(rf"(?<!{WORD_CHAR}){re.escape(span)}(?!{WORD_CHAR})")
    matches = list(pattern.finditer(source))
    if len(matches) != 1:
        return None
    before, after = matches[0].span()
    masked = source[:before] + GAP + source[after:]
    return None if shows_target(masked, target) else masked


def gapped(source: str | None, target: str, span: str = "") -> str | None:
    """The source with its span realizing the target, even inflected, replaced by a gap."""
    if not source or GAP in source:
        return None
    return (_gap(source, span, target) if span.strip() else None) or _gap(source, target, target)


def _hidden(letters: str) -> str:
    return "".join("_" if character.isalpha() else character for character in letters)


def _fragment(word: str) -> str:
    # A one-letter word shows only its slot; showing it would show the whole word.
    if len(word) < 2:  # noqa: PLR2004
        return "_"
    if len(word) >= LONG_WORD:
        head, tail = min(3, len(word) // 3), 2
    else:
        head, tail = 1, 1 if len(word) >= TAIL_LETTER_WORD else 0
    return word[:head] + _hidden(word[head : len(word) - tail]) + word[len(word) - tail :]


def partial_form(target: str) -> str | None:
    """Each word shortened in place with one _ per hidden letter, such as o_ t__ f___e."""
    partial = TARGET_WORD.sub(lambda word: _fragment(word.group()), target)
    if not any(character.isalpha() for character in partial):
        return None
    return None if contains_target(partial, target) else partial


def basic_card(
    target: str,
    original: str,
    source: str | None,
    native_language: str,
    span: str = "",
) -> BasicCard | None:
    """Wording practice from the confirmed target; None when no cue can conceal the answer."""
    instruction = message("instruction", native_language)
    masked = gapped(source, target, span)
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


def confirmed_card(
    target: str,
    original: str,
    source: str | None,
    span: str,
    native_language: str,
) -> BasicCard:
    """Basic practice for a confirmed target; when none fits, the reason is a decision."""
    # A source too long for a card still leaves the target to practise on its own.
    for candidate in [source, None] if source else [None]:
        try:
            card = basic_card(target, original, candidate, native_language, span)
        except ValidationError as error:
            if all(problem["type"] == "string_too_long" for problem in error.errors()):
                continue
            card = None
        if card is None:
            raise PreparationError(message("no_safe_cue", native_language), retry_helps=False)
        return card
    raise PreparationError(message("too_long", native_language), retry_helps=False)
