from typing import Any

import pytest

from phrase_gym.coach.checked_preparation import PreparedPhrase, checked_card
from phrase_gym.coach.drafts import Check, Draft, parse_check, parse_draft
from phrase_gym.coach.preparation import PreparationError
from phrase_gym.server.coach_contracts import BasicCard, CoachWork

SOURCE = "We strongly believe it works."
TYPED = f"strongly believe — {SOURCE}"


def recognition(**changes: object) -> dict[str, object]:
    return {
        "outcome": "accept",
        "original_span": "strongly believe",
        "target": "strongly believe",
        "source": SOURCE,
        "source_usable": True,
        "kind": "phrase",
        "reason": "",
        **changes,
    }


def example(text: str, attribution: str = "generated", gloss: str = "truly think") -> dict:
    return {"text": text, "gloss": gloss, "partner_line": "", "attribution": attribution}


def written(**changes: Any) -> dict[str, Any]:
    return {
        "recognition": recognition(),
        "explanation": "**strongly believe**: be very sure that something is true.",
        "examples": [
            example(SOURCE, "source", "are sure"),
            example("I strongly believe she will win.", gloss="am convinced"),
        ],
        "recall_hints": ["str... / opinion", "bel... / values"],
        "usage_contexts": ["stating a firm opinion"],
        **changes,
    }


def judged(draft: dict[str, Any], **changes: Any) -> dict[str, Any]:
    return {
        "recognition": recognition(),
        "explanation_ok": True,
        "example_ok": [True] * len(draft["examples"]),
        "hints_ok": [True] * len(draft["recall_hints"]),
        "usage_contexts_ok": [True] * len(draft["usage_contexts"]),
        "reasons": [],
        **changes,
    }


def work(text: str = TYPED) -> CoachWork:
    return CoachWork(
        package_id="coach-1",
        text=text,
        learning_language="en",
        card_language="en",
        help_language="en",
    )


def card(draft: dict[str, Any], check: dict[str, Any]) -> BasicCard | PreparedPhrase:
    parsed: Draft = parse_draft(draft, TYPED)
    checked: Check = parse_check(check, parsed, TYPED)
    return checked_card(parsed, checked, work())


def cues(result: BasicCard | PreparedPhrase) -> list[str]:
    assert isinstance(result, PreparedPhrase)
    return [cue.situation for cue in result.phrase.cues]


def test_checked_additions_become_cues_with_the_source_revealed() -> None:
    result = card(written(), judged(written()))
    assert cues(result) == [
        "We __________ it works. (=are sure)",
        "I __________ she will win. (=am convinced)",
    ]
    assert isinstance(result, PreparedPhrase) and result.phrase.text == SOURCE


def test_a_hint_failing_the_shared_measured_limits_is_dropped_on_its_own() -> None:
    draft = written(recall_hints=["str... / opinion", "strongl... / values", "bel... / values"])
    result = card(draft, judged(draft))
    assert isinstance(result, PreparedPhrase)
    assert result.phrase.recall_hints == ["str... / opinion", "bel... / values"]


@pytest.mark.parametrize(
    ("examples", "kept"),
    [
        (
            [example("We strongly believe it works!", "source"), example("I strongly believe it.")],
            "I __________ it. (=truly think)",
        ),
        (
            [example(SOURCE), example("I strongly believe she will win.")],
            "I __________ she will win. (=truly think)",
        ),
    ],
    ids=["rewritten source called a quote", "the source called new"],
)
def test_an_example_with_false_attribution_is_dropped(examples: list[dict], kept: str) -> None:
    draft = written(examples=examples)
    assert cues(card(draft, judged(draft))) == [kept]


def test_disagreement_about_the_kind_or_the_source_keeps_safer_practice() -> None:
    kind = card(written(), judged(written(), recognition=recognition(kind="word")))
    assert isinstance(kind, BasicCard) and kind.cue == "We __________ it works."
    doubted = recognition(source_usable=False)
    alone = card(written(), judged(written(), recognition=doubted))
    assert isinstance(alone, PreparedPhrase)
    assert alone.phrase.text == "I strongly believe she will win."
    assert cues(alone) == ["I __________ she will win. (=am convinced)"]


def test_a_rejection_by_both_calls_keeps_the_checker_reason() -> None:
    rejected = recognition(
        outcome="reject", original_span="", target="", source="", source_usable=False
    )
    draft = written(recognition={**rejected, "reason": "Not English."}, examples=[])
    draft.update(explanation="", recall_hints=[], usage_contexts=[])
    check = judged(draft, recognition={**rejected, "reason": "This is not English."})
    with pytest.raises(PreparationError, match="This is not English.") as raised:
        card(draft, check)
    assert not raised.value.retry_helps


@pytest.mark.parametrize(
    ("answer", "problem"),
    [
        ({**written(), "extra": 1}, "extra: Extra inputs are not permitted"),
        (written(recognition=recognition(source_usable="yes")), "source_usable"),
        (written(recognition=recognition(original_span="firmly")), "cannot be traced"),
    ],
)
def test_a_malformed_draft_is_a_service_failure(answer: dict[str, Any], problem: str) -> None:
    with pytest.raises(PreparationError, match=problem) as raised:
        parse_draft(answer, TYPED)
    assert raised.value.retry_helps
