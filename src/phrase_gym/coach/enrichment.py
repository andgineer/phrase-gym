import re
from typing import Any

from phrase_gym.coach.basic_card import contains_target, gapped
from phrase_gym.coach.card_checks import CUE_COUNTS, card_problems, phrase_of
from phrase_gym.coach.drafts import Check, Draft, Recognition
from phrase_gym.server.import_contracts import (
    MAX_CONTEXT_LENGTH,
    MAX_HINT_LENGTH,
    ImportPhrase,
)

DROPPABLE = re.compile(r"^(cues|recall_hints|usage_contexts)\.(\d+)\b")
MAX_ITEMS = 6


def _verified_texts(
    values: list[str],
    flags: list[bool],
    limit: int,
    hidden: str | None = None,
) -> list[str]:
    kept: dict[str, str] = {}
    for raw, accepted in zip(values, flags, strict=True):
        value = raw.strip()
        shows_answer = hidden is not None and contains_target(value, hidden)
        if accepted and 0 < len(value) <= limit and not shows_answer:
            kept.setdefault(value.casefold(), value)
    return list(kept.values())[:MAX_ITEMS]


def _verified_cues(
    draft: Draft,
    check: Check,
    recognition: Recognition,
    source: str | None,
) -> tuple[list[dict[str, Any]], list[str | None]]:
    """Each checked example as a gapped cue, beside its complete text when Coach wrote it."""
    target = recognition.target
    cues: list[dict[str, Any]] = []
    written: list[str | None] = []
    seen: set[str] = set()
    for example, accepted in zip(draft.examples, check.example_ok, strict=True):
        text, gloss = example.text.strip(), example.gloss.strip()
        masked = gapped(text, target)
        copied = bool(recognition.source) and text == recognition.source
        # A quote must be the usable source verbatim; a new example must not copy the source.
        if example.attribution == "source":
            attributed = source is not None and copied
        else:
            attributed = not copied
        if not (accepted and masked and gloss and attributed) or contains_target(gloss, target):
            continue
        if text.casefold() in seen:
            continue
        seen.add(text.casefold())
        partner = example.partner_line.strip() or None
        cues.append({"situation": f"{masked} (={gloss})", "partner_line": partner})
        written.append(None if copied else text)
    return cues, written


def _drop_first_flagged(
    card: dict[str, Any],
    pool: tuple[list[dict[str, Any]], list[str | None]],
    problems: list[str],
) -> bool:
    """Remove one named addition, since it may be what makes another look wrong."""
    found = next((match for match in map(DROPPABLE.match, problems) if match), None)
    if found is None:
        return False
    field, index = found[1], int(found[2])
    if field == "cues":
        # Shown cues lead the pool, so the next approved spare takes the dropped cue's place.
        for candidates in pool:
            del candidates[index]
    else:
        del card[field][index]
    return True


def enriched_phrase(
    draft: Draft,
    check: Check,
    recognition: Recognition,
    source: str | None,
    item_id: str,
) -> ImportPhrase | None:
    """The checked additions that also pass the shared mechanical rules, or None for basic."""
    target = recognition.target
    if not check.explanation_ok or not draft.explanation.strip():
        return None
    cues, written = _verified_cues(draft, check, recognition, source)
    high = CUE_COUNTS[recognition.kind][1]
    card: dict[str, Any] = {
        "explanation": draft.explanation.strip(),
        "recall_hints": _verified_texts(
            draft.recall_hints,
            check.hints_ok,
            MAX_HINT_LENGTH,
            hidden=target,
        ),
        "usage_contexts": _verified_texts(
            draft.usage_contexts,
            check.usage_contexts_ok,
            MAX_CONTEXT_LENGTH,
        ),
        "detailed_explanation": None,
    }
    answer = {"kind": recognition.kind, "target": target, "card": card}
    # Each round drops one addition, so the number of additions bounds the rounds.
    for _ in range(1 + len(cues) + len(card["recall_hints"]) + len(card["usage_contexts"])):
        new = [text for text in written[:high] if text]
        if not new:
            # Without a new context the card would only repeat basic practice of the source.
            return None
        card["cues"] = cues[:high]
        card["text"] = source if source and gapped(source, target) else new[0]
        problems = card_problems(answer, item_id)
        if not problems:
            return phrase_of(card, item_id)
        if not _drop_first_flagged(card, (cues, written), problems):
            return None
    return None
