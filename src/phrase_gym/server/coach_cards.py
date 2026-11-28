import json

from pydantic import ValidationError

from phrase_gym.server.coach_contracts import (
    COACH_LESSON_ID,
    COACH_SOURCE_ID,
    COACH_SOURCE_LABEL,
    BasicCard,
    CoachCorrection,
    CoachResult,
    CoachWork,
)
from phrase_gym.server.import_contracts import ImportLesson, ImportPhrase, PhrasePackage
from phrase_gym.server.languages import NativeLanguage
from phrase_gym.server.publication import Batch, BatchPhrase, package_batch
from phrase_gym.server.reference_audio import (
    ImportConflictError,
    decode_reference,
    decode_references,
)
from phrase_gym.server.search_terms import search_key


def coach_package(work: CoachWork, phrase: ImportPhrase) -> PhrasePackage:
    return PhrasePackage(
        format="phrase-gym.package",
        version=3,
        package_id=work.package_id,
        source_id=COACH_SOURCE_ID,
        source_label=COACH_SOURCE_LABEL,
        learning_language=work.learning_language,
        meaning_language=work.native_language if work.translation else work.learning_language,
        lessons=[
            ImportLesson(lesson_id=COACH_LESSON_ID, title=COACH_SOURCE_LABEL, phrases=[phrase]),
        ],
    )


# Like the card's instruction, the note speaks the native language captured with the request.
CORRECTION_NOTES: dict[NativeLanguage, str] = {
    "en": "Corrected from “{}”.",
    "ru": "Исправлено, было: «{}».",
}


def correction_note(correction: CoachCorrection | None, language: NativeLanguage) -> str:
    return CORRECTION_NOTES[language].format(correction.original) if correction else ""


def _noted(phrase: ImportPhrase, note: str) -> ImportPhrase:
    if not note:
        return phrase
    try:
        return ImportPhrase.model_validate(
            {**phrase.model_dump(), "explanation": f"{phrase.explanation} {note}"},
        )
    except ValidationError as error:
        raise ImportConflictError(
            "The explanation leaves no room for the correction note.",
        ) from error


def _basic_content(card: BasicCard, language: NativeLanguage) -> dict[str, object]:
    note = correction_note(card.correction, language)
    return {
        "source_label": COACH_SOURCE_LABEL,
        "source_link": None,
        "text": card.text,
        "explanation": note,
        "detailed_explanation": None,
        "cues_json": json.dumps([{"situation": card.cue}]),
        "hints_json": "[]",
        "contexts_json": "[]",
        "variants_json": "[]",
        "search_primary": search_key(card.text),
        "search_text": search_key(card.text, card.target, note, card.cue),
        "instruction": card.instruction,
    }


def coach_batch(work: CoachWork, result: CoachResult) -> Batch:
    package_id = work.package_id
    correction = result.corrected()
    if correction and search_key(correction.original) not in search_key(work.text):
        raise ImportConflictError(
            "The corrected original must be whole words of the submitted text.",
        )
    if result.phrase is not None:
        if result.phrase.item_id != package_id:
            raise ImportConflictError(f"phrases.item_id must be {package_id}.")
        note = correction_note(result.correction, work.native_language)
        package = coach_package(work, _noted(result.phrase, note))
        return package_batch(package, decode_references(package, {package_id: result.reference}))
    if result.basic is None:
        raise ImportConflictError("A Coach result needs a card.")
    return Batch(
        package_id=package_id,
        source_id=COACH_SOURCE_ID,
        source_label=COACH_SOURCE_LABEL,
        language=work.learning_language,
        lessons=[(COACH_LESSON_ID, COACH_SOURCE_LABEL)],
        phrases=[
            BatchPhrase(
                COACH_LESSON_ID,
                package_id,
                _basic_content(result.basic, work.native_language),
                decode_reference(package_id, result.reference),
            ),
        ],
    )
