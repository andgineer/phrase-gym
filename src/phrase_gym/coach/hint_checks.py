import re

from phrase_gym.server.import_contracts import Cue

ELLIPSIS = re.compile(r"\.\.\.|…")
WORD = re.compile(r"[\w']+")
GLOSS = re.compile(r"\(=([^)]*)\)")
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


def hint_problems(index: int, hint: str, target_text: str, cues: list[Cue]) -> list[str]:
    target = [letters(word) for word in words(target_text) if letters(word)]
    free = list(range(len(target)))
    problems: list[str] = []
    shown_total = 0
    for chunk in re.split(r"[/,]", hint):
        companions = []
        for token in chunk.split():
            if not letters(token):
                continue
            matched = _match(token, target, free)
            if matched is None:
                companions.append(token)
                continue
            word, shown = matched
            free.remove(word)
            shown_total += shown
            if shown != len(target[word]) and shown * 2 > len(target[word]):
                problems.append(
                    f"recall_hints.{index}: “{token}” shows {shown} of the {len(target[word])} "
                    "letters of a target word; show at most half",
                )
        companion = words(" ".join(companions))
        visible = [f"{cue.situation} {cue.partner_line or ''}" for cue in cues]
        if companion and any(contains_words(text, companion) for text in visible):
            problems.append(
                f"recall_hints.{index}: “{' '.join(companions)}” is already visible in a cue",
            )
    total = sum(len(word) for word in target)
    if total and total - shown_total <= total * MIN_HIDDEN_SHARE:
        problems.append(
            f"recall_hints.{index}: the hint leaves only {total - shown_total} of the target's "
            f"{total} letters hidden; keep more than a quarter hidden",
        )
    return problems


def gloss_problems(cues: list[Cue], target_text: str, variants: list[str]) -> list[str]:
    problems = []
    for index, cue in enumerate(cues):
        for gloss in GLOSS.findall(cue.situation):
            for answer in [target_text, *variants]:
                if words(answer) and contains_words(gloss, words(answer)):
                    problems.append(
                        f"cues.{index}: the gloss contains the answer “{answer}”; "
                        "paraphrase it instead",
                    )
    return problems
