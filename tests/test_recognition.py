import pytest

from phrase_gym.coach.basic_card import basic_card, gapped, partial_form
from phrase_gym.coach.preparation import PreparationError
from phrase_gym.coach.recognition import recognition_problem, recognized_card


@pytest.mark.parametrize(
    ("target", "partial"),
    [
        ("definitely", "def…ly"),
        ("strongly believe", "st…ly"),
        ("colour", "co…"),
        ("yet", "y…"),
        ("I", None),
    ],
)
def test_a_partial_form_keeps_most_of_the_longest_word_hidden(
    target: str, partial: str | None
) -> None:
    assert partial_form(target) == partial


@pytest.mark.parametrize(
    ("source", "target", "cue"),
    [
        ("We strongly believe this will work", "strongly believe", "We __________ this will work"),
        ("Yes, I believe so.", "believe", "Yes, I __________ so."),
        ("I believe and believe again.", "believe", None),
        ("We believed it.", "believe", None),
        ("We think so.", "believe", None),
        (None, "believe", None),
    ],
)
def test_a_gap_replaces_exactly_one_whole_target_span(
    source: str | None, target: str, cue: str | None
) -> None:
    assert gapped(source, target) == cue


def test_a_card_reveals_the_source_only_when_its_gap_works() -> None:
    gap = basic_card("strongly believe", "strongly believe", "We strongly believe it works.")
    assert gap and (gap.text, gap.cue) == (
        "We strongly believe it works.",
        "We __________ it works.",
    )
    alone = basic_card("believe", "belive", "We believed it.")
    assert alone and (alone.text, alone.cue) == ("believe", "be…ve")
    assert alone.correction and alone.correction.original == "belive"
    assert basic_card("I", "I", None) is None


def answer(**changes: object) -> dict[str, object]:
    return {
        "outcome": "accept",
        "original_span": "strongly believe",
        "target": "strongly believe",
        "source": "",
        "source_usable": False,
        "reason": "",
        **changes,
    }


@pytest.mark.parametrize(
    ("changes", "problem"),
    [
        ({"outcome": "maybe"}, "The recognition answer has no outcome"),
        ({"source_usable": "yes"}, "missing required fields"),
        ({"original_span": "firmly believe"}, "cannot be traced to the submitted text"),
        ({"source": "We firmly believe."}, "source sentence cannot be traced"),
        ({"source_usable": True}, "confirms a source that was not supplied"),
        ({"target": " strongly believe"}, "lacks the original span or the confirmed target"),
        ({"outcome": "reject", "target": "", "reason": " "}, "The rejection gives no reason"),
    ],
)
def test_malformed_recognition_cannot_publish_or_reject(
    changes: dict[str, object], problem: str
) -> None:
    found = recognition_problem(answer(**changes), "strongly believe")
    assert found is not None and problem in found
    with pytest.raises(PreparationError) as raised:
        recognized_card(answer(**changes), "strongly believe")
    assert raised.value.retry_helps


def test_a_rejection_is_a_language_decision() -> None:
    rejected = answer(outcome="reject", original_span="", target="", reason="Not English.")
    with pytest.raises(PreparationError, match="Not English.") as raised:
        recognized_card(rejected, "zzqx")
    assert not raised.value.retry_helps
