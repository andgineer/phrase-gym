import re

from phrase_gym.server.search_terms import search_key

GAP = "__________"
# A longer underscore run is a partial form of one long word, not a sentence gap.
STANDALONE_GAP = re.compile(rf"(?<!\w){re.escape(GAP)}(?!\w)")
PREFIX_LETTERS = 3


def _words(text: str) -> list[str]:
    return search_key(text).split()


def gap_around(cue: str) -> tuple[str, str] | None:
    """The cue text before and after its one sentence gap, if it has exactly one."""
    gaps = list(STANDALONE_GAP.finditer(cue))
    if len(gaps) != 1:
        return None
    return cue[: gaps[0].start()], cue[gaps[0].end() :]


def _realizes(found: list[str], wanted: list[str]) -> bool:
    # An inflected form keeps each word's first letter: came/come, took/take, got/get.
    return (
        bool(wanted)
        and len(found) == len(wanted)
        and all(a == b or a[0] == b[0] for a, b in zip(found, wanted, strict=True))
    )


def realizes(span: str, target: str) -> bool:
    """Whether a sentence span can be a form of the target, such as came up with."""
    return _realizes(_words(span), _words(target))


def _shared_prefix(a: str, b: str) -> bool:
    length = min(PREFIX_LETTERS, len(a), len(b))
    return a[:length] == b[:length]


def shows_target(text: str, target: str) -> bool:
    """Whether the text still shows the target or a recognizable form of it, such as comes up."""
    found, wanted = _words(text), _words(target)
    for start in range(len(found) - len(wanted) + 1):
        window = found[start : start + len(wanted)]
        if not _realizes(window, wanted):
            continue
        pairs = list(zip(window, wanted, strict=True))
        if any(a == b for a, b in pairs) or all(_shared_prefix(a, b) for a, b in pairs):
            return True
    return False
