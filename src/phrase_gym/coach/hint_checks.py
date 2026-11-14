import re
from dataclasses import dataclass, field

from phrase_gym.server.import_contracts import Cue

ELLIPSIS = re.compile(r"\.\.\.|…")
WORD = re.compile(r"[\w']+")
GLOSS = re.compile(r"\(=([^)]*)\)")
GAP_TEXT = "__________"
BOLD = re.compile(r"\*\*(.+?)\*\*")
MIN_OPENING_WORDS = 2
# Words that dangle at a hint's end; prepositions such as "to" or "about" may still end one.
FUNCTION_WORDS = frozenset(
    [
        "a",
        "an",
        "the",
        "and",
        "or",
        "but",
        "my",
        "your",
        "his",
        "her",
        "its",
        "our",
        "their",
        "this",
        "that",
        "these",
        "those",
        "some",
        "any",
    ],
)
# A shortened word may show at most half its letters; over a quarter of the target stays hidden.
MIN_HIDDEN_SHARE = 0.25


def words(text: str) -> list[str]:
    return [word.casefold() for word in WORD.findall(text.replace("’", "'"))]


def contains_words(text: str, target: list[str]) -> bool:
    found = words(text)
    return any(found[index : index + len(target)] == target for index in range(len(found)))


def letters(text: str) -> str:
    return re.sub(r"[^a-z]", "", text.casefold())


def _fragment(token: str) -> tuple[re.Pattern[str], int] | None:
    pieces = [letters(piece) for piece in ELLIPSIS.split(token)]
    shown = [piece for piece in pieces if piece]
    if not shown:
        return None
    if len(pieces) == 1:
        # A shortened word counts even without "...": any proper part of a target word.
        return (
            (re.compile(f".*{re.escape(shown[0])}.*"), len(shown[0])) if len(shown[0]) > 1 else None
        )
    body = ".+".join(re.escape(piece) for piece in shown)
    pattern = (".+" if not pieces[0] else "") + body + (".+" if not pieces[-1] else "")
    return re.compile(pattern), sum(len(piece) for piece in shown)


def _match(token: str, target: list[str], free: list[int]) -> tuple[int, int] | None:
    whole = letters(token)
    if not ELLIPSIS.search(token):
        found = next((index for index in free if whole and target[index] == whole), None)
        if found is not None:
            return found, len(whole)
    fragment = _fragment(token)
    if fragment is None:
        return None
    pattern, shown = fragment
    for index in free:
        if shown < len(target[index]) and pattern.fullmatch(target[index]):
            return index, shown
    return None


@dataclass
class HintParts:
    whole_words: list[int] = field(default_factory=list)
    shown: int = 0
    companions: list[list[str]] = field(default_factory=list)
    problems: list[str] = field(default_factory=list)
    dangling: list[str] = field(default_factory=list)


def parse_hint(index: int, hint: str, target: list[str]) -> HintParts:
    parts = HintParts()
    free = list(range(len(target)))
    for chunk in re.split(r"[/,]", hint):
        companions: list[str] = []
        tokens = [token for token in chunk.split() if letters(token)]
        for token in tokens:
            matched = _match(token, target, free)
            if matched is None:
                companions.append(token)
                continue
            # Function words right before a target word belong to it ("make a pi...").
            while companions and _bare(companions[-1]):
                companions.pop()
            word, shown = matched
            free.remove(word)
            parts.shown += shown
            if shown == len(target[word]):
                parts.whole_words.append(word)
            elif shown * 2 > len(target[word]):
                parts.problems.append(
                    f"recall_hints.{index}: “{token}” shows {shown} of the {len(target[word])} "
                    "letters of a target word; show at most half",
                )
        parts.companions.append(companions)
        if tokens and companions and tokens[-1] == companions[-1] and _bare(tokens[-1]):
            parts.dangling.append(" ".join(companions))
    return parts


def _bare(token: str) -> bool:
    return token.casefold().replace("’", "'").strip("'\"") in FUNCTION_WORDS


def hint_problems(index: int, hint: str, target_text: str, cues: list[Cue]) -> list[str]:
    target = [letters(word) for word in words(target_text) if letters(word)]
    parts = parse_hint(index, hint, target)
    problems = parts.problems
    visible = [f"{cue.situation} {cue.partner_line or ''}" for cue in cues]
    for companions in parts.companions:
        companion = words(" ".join(companions))
        if companion and any(contains_words(text, companion) for text in visible):
            problems.append(
                f"recall_hints.{index}: “{' '.join(companions)}” is already visible in a cue",
            )
    problems += [
        f"recall_hints.{index}: “{chunk}” ends with the bare word “{chunk.split()[-1]}”; "
        "end on a word that carries meaning"
        for chunk in parts.dangling
    ]
    total = sum(len(word) for word in target)
    if total and total - parts.shown <= total * MIN_HIDDEN_SHARE:
        problems.append(
            f"recall_hints.{index}: the hint leaves only {total - parts.shown} of the target's "
            f"{total} letters hidden; keep more than a quarter hidden",
        )
    opening = parts.whole_words
    if len(opening) >= MIN_OPENING_WORDS and opening == list(range(len(opening))):
        problems.append(
            f"recall_hints.{index}: the hint is the target's opening words in order; "
            "anchor on words from different parts of it",
        )
    return problems


def bold_expressions(*texts: str | None) -> list[str]:
    return [found.strip() for text in texts if text for found in BOLD.findall(text)]


def hint_set_problems(hints: list[str], target_text: str, contrasted: list[str]) -> list[str]:
    problems = []
    for index, hint in enumerate(hints):
        for expression in contrasted:
            other = words(expression)
            own = set(words(target_text)) | FUNCTION_WORDS
            if other and not set(other) <= own and contains_words(hint, other):
                problems.append(
                    f"recall_hints.{index}: it contains “{expression}”, the expression this "
                    "card contrasts; point to the target instead",
                )
    return problems


def gloss_problems(cues: list[Cue], target_text: str) -> list[str]:
    target = words(target_text)
    return [
        f"cues.{index}: the gloss contains the answer “{target_text}”; paraphrase it instead"
        for index, cue in enumerate(cues)
        for gloss in GLOSS.findall(cue.situation)
        if target and contains_words(gloss, target)
    ]
