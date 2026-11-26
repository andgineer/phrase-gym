import pytest

from phrase_gym.coach.basic_card import basic_card, gapped, partial_form
from phrase_gym.coach.messages import message
from phrase_gym.coach.preparation import PreparationError
from phrase_gym.coach.recognition import recognition_problem, recognized_card
from phrase_gym.server.coach_contracts import CoachWork


def work(text: str, native_language: str = "en") -> CoachWork:
    return CoachWork(
        package_id="coach-test",
        text=text,
        learning_language="en",
        native_language=native_language,
        translation=False,
    )


@pytest.mark.parametrize(
    ("target", "partial"),
    [
        ("colour", "c____r"),
        ("their", "t___r"),
        ("reluctant", "rel____nt"),
        ("definitely", "def_____ly"),
        ("on the fence", "o_ t__ f___e"),
        ("come up with", "c__e u_ w__h"),
        ("strongly believe", "st____ly be___ve"),
        ("take it for granted", "t__e i_ f__ gr___ed"),
        ("a lot", "_ l__"),
        ("don't mind", "d__'t m__d"),
        ("misunderstanding", "mis___________ng"),
        ("responsibilities", "res___________es"),
        ("straightforward", "str__________rd"),
        ("I", None),
    ],
)
def test_a_partial_form_keeps_every_word_in_order_without_showing_any(
    target: str, partial: str | None
) -> None:
    assert partial_form(target) == partial


@pytest.mark.parametrize(
    ("source", "target", "cue"),
    [
        ("We strongly believe this will work", "strongly believe", "We __________ this will work"),
        ("Yes, I believe so.", "believe", "Yes, I __________ so."),
        ("I believe and believe again.", "believe", None),
        ("She came up with a plan.", "come up with", None),
        ("We believed it.", "believe", None),
        ("We think so.", "believe", None),
        (None, "believe", None),
    ],
)
def test_a_gap_replaces_exactly_one_whole_target_span(
    source: str | None, target: str, cue: str | None
) -> None:
    assert gapped(source, target) == cue


@pytest.mark.parametrize(
    ("span", "cue"),
    [
        ("came up with", "She __________ a brilliant plan overnight."),
        ("come up with", None),
        ("came", None),
        ("up with a", None),
        ("a brilliant plan", None),
        ("", None),
    ],
)
def test_the_span_realizing_the_target_is_gapped_even_when_inflected(
    span: str, cue: str | None
) -> None:
    source = "She came up with a brilliant plan overnight."
    assert gapped(source, "come up with", span) == cue


def test_a_span_found_twice_falls_back_to_the_literal_target() -> None:
    source = "She came up with it, then came up with more."
    assert gapped(source, "come up with", "came up with") is None
    leaky = "She came up with a plan to come up with more."
    assert gapped(leaky, "come up with", "came up with") is None
    assert gapped("We believe it, we believe.", "believe", "believe") is None
    assert gapped("Yes, I believe so.", "believe", "believed") == "Yes, I __________ so."


def test_a_target_that_is_its_whole_source_gets_a_partial_form() -> None:
    assert gapped("Wie geht's?", "Wie geht's?", "Wie geht's?") is None
    card = basic_card("Wie geht's?", "Wie geht's?", "Wie geht's?", "ru", "Wie geht's?")
    assert card and (card.text, card.cue) == ("Wie geht's?", "W__ g___'s?")
    assert card.instruction == "Произнесите слово или фразу целиком."


def test_a_gap_is_refused_while_another_form_of_the_target_stays_visible() -> None:
    source = "She came up with a plan, and he comes up with another."
    assert gapped(source, "come up with", "came up with") is None


@pytest.mark.parametrize("target", ["misunderstanding", "responsibilities", "straightforward"])
def test_a_long_word_gets_a_partial_form_card(target: str) -> None:
    card = basic_card(target, target, None, "en")
    assert card and card.text == target and target[:3] in card.cue


def test_a_card_reveals_the_source_only_when_its_gap_works() -> None:
    gap = basic_card("strongly believe", "strongly believe", "We strongly believe it works.", "en")
    assert gap and (gap.text, gap.cue) == (
        "We strongly believe it works.",
        "We __________ it works.",
    )
    alone = basic_card("believe", "belive", "We believed it.", "en")
    assert alone and (alone.text, alone.cue) == ("believe", "be___ve")
    assert alone.correction and alone.correction.original == "belive"
    assert basic_card("I", "I", None, "en") is None


def test_an_inflected_sentence_gap_reveals_the_sentence_and_studies_the_target() -> None:
    source = "She came up with a brilliant plan overnight."
    card = basic_card("come up with", "come up with", source, "en", "came up with")
    assert card and (card.target, card.text, card.cue) == (
        "come up with",
        source,
        "She __________ a brilliant plan overnight.",
    )


@pytest.mark.parametrize(
    ("language", "instruction"),
    [("en", "Say the missing wording."), ("ru", "Произнесите пропущенное.")],
)
def test_the_instruction_is_in_the_captured_native_language(
    language: str, instruction: str
) -> None:
    sentence = "We strongly believe it works."
    found = answer(source=sentence, source_span="strongly believe", source_usable=True)
    card = recognized_card(found, work(f"strongly believe — {sentence}", language))
    assert (card.cue, card.instruction) == ("We __________ it works.", instruction)


@pytest.mark.parametrize(
    ("language", "instruction"),
    [("en", "Say the whole word or phrase."), ("ru", "Произнесите слово или фразу целиком.")],
)
def test_a_partial_form_asks_for_the_whole_wording(language: str, instruction: str) -> None:
    card = recognized_card(answer(), work("strongly believe", language))
    assert (card.cue, card.instruction) == ("st____ly be___ve", instruction)


def answer(**changes: object) -> dict[str, object]:
    return {
        "outcome": "accept",
        "original_span": "strongly believe",
        "target": "strongly believe",
        "source": "",
        "source_span": "",
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
        recognized_card(answer(**changes), work("strongly believe"))
    assert raised.value.retry_helps


def test_a_rejection_is_a_language_decision() -> None:
    rejected = answer(outcome="reject", original_span="", target="", reason="Not English.")
    with pytest.raises(PreparationError, match="Not English.") as raised:
        recognized_card(rejected, work("zzqx"))
    assert not raised.value.retry_helps


def test_a_source_too_long_for_a_card_leaves_the_target_alone() -> None:
    source = "We strongly believe " + "and so on " * 60 + "it."
    accepted = answer(source=source, source_span="strongly believe", source_usable=True)
    card = recognized_card(accepted, work(f"strongly believe — {source}"))
    assert (card.text, card.cue) == ("strongly believe", "st____ly be___ve")


@pytest.mark.parametrize("language", ["en", "ru"])
def test_a_target_too_long_for_a_card_says_so_in_the_native_language(language: str) -> None:
    target = " ".join(["strongly"] * 30)
    with pytest.raises(PreparationError) as raised:
        recognized_card(answer(original_span=target, target=target), work(target, language))
    assert str(raised.value) == message("too_long", language)
    assert not raised.value.retry_helps


@pytest.mark.parametrize("language", ["en", "ru"])
def test_a_target_without_a_hiding_cue_says_so_in_the_native_language(language: str) -> None:
    with pytest.raises(PreparationError) as raised:
        recognized_card(answer(original_span="I", target="I"), work("I", language))
    assert str(raised.value) == message("no_safe_cue", language)
    assert not raised.value.retry_helps


def test_inbox_reasons_have_english_and_russian_texts() -> None:
    assert message("too_long", "en") == "The text is too long for one card."
    assert message("too_long", "ru") == "Текст слишком длинный для одной карточки."
    assert message("no_safe_cue", "ru") != message("no_safe_cue", "en")
