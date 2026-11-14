from phrase_gym.coach.hint_checks import (
    FUNCTION_WORDS,
    GAP_TEXT,
    GLOSS,
    contains_words,
    letters,
    parse_hint,
    words,
)
from phrase_gym.server.import_contracts import Cue

# Small grammatical words never count as the substance a companion shares or borrows.
MINOR_WORDS = FUNCTION_WORDS | frozenset(
    [
        "to",
        "of",
        "for",
        "with",
        "in",
        "on",
        "at",
        "by",
        "from",
        "about",
        "into",
        "over",
        "up",
        "out",
        "me",
        "you",
        "him",
        "her",
        "us",
        "them",
        "it",
        "i",
        "we",
        "they",
        "is",
        "are",
        "be",
        "do",
    ],
)
# Words that are also verb particles ("carry on", "get on", "take over") are left out, because a
# phrasal-verb gloss may legitimately end on one before a prepositional continuation.
FINAL_PREPOSITIONS = frozenset(["about", "of", "for", "to", "with", "at", "by", "from", "into"])
PREPOSITIONS = FINAL_PREPOSITIONS | {"on", "in", "over"}


def _content(companions: list[str]) -> set[str]:
    return {word for word in words(" ".join(companions)) if word.strip("'") not in MINOR_WORDS}


def _companions(hints: list[str], target_text: str) -> list[list[str]]:
    target = [letters(word) for word in words(target_text) if letters(word)]
    return [sum(parse_hint(index, hint, target).companions, []) for index, hint in enumerate(hints)]


def companion_problems(hints: list[str], target_text: str, cues: list[Cue], text: str) -> list[str]:
    problems = []
    companions = _companions(hints, target_text)
    glosses = [gloss for cue in cues for gloss in GLOSS.findall(cue.situation) if words(gloss)]
    seen: dict[str, int] = {}
    for index, own in enumerate(companions):
        phrase = words(" ".join(own))
        for gloss in glosses:
            if len(phrase) > 1 and (
                contains_words(gloss, phrase) or contains_words(" ".join(phrase), words(gloss))
            ):
                problems.append(
                    f"recall_hints.{index}: “{' '.join(own)}” repeats a cue gloss; "
                    "use a collocate, situation or sentence frame instead",
                )
        for word in sorted(_content(own)):
            if word in seen:
                problems.append(
                    f"recall_hints.{index}: it shares “{word}” with recall_hints.{seen[word]}; "
                    "give each hint different companion words",
                )
            seen.setdefault(word, index)
    used = [_content(own) for own in companions if _content(own)]
    source = set(words(text))
    if len(hints) > 1 and len(used) == len(hints) and all(found <= source for found in used):
        problems.append(
            "recall_hints: every companion comes from the example sentence; "
            "take at least one from a different context",
        )
    return problems


def gloss_frame_problems(cues: list[Cue]) -> list[str]:
    problems = []
    for index, cue in enumerate(cues):
        tail = cue.situation.split(GAP_TEXT, 1)[-1] if GAP_TEXT in cue.situation else ""
        after = words(GLOSS.sub(" ", tail))
        for gloss in GLOSS.findall(cue.situation):
            last = words(gloss)[-1:] if words(gloss) else []
            if last and after and last[0] in FINAL_PREPOSITIONS and after[0] in PREPOSITIONS:
                problems.append(
                    f"cues.{index}: the gloss ends with “{last[0]}” before “{after[0]}” in the "
                    "cue; make it fit the sentence frame",
                )
    return problems
