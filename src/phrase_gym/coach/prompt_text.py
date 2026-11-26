import json
import re
from importlib.resources import files
from typing import Any

from phrase_gym.server.coach_contracts import CoachWork
from phrase_gym.server.languages import LearningLanguage

PROMPTS = files("phrase_gym.coach").joinpath("prompts")
PLACEHOLDER = re.compile(r"\{\{(\w+)\}\}")
# Prompts name languages in English; the answers are written in the named language.
LANGUAGE_NAMES = {"en": "English", "de": "German", "ru": "Russian"}
# The learner's stated levels from the language choices; Settings records no proficiency.
STATED_LEVELS: dict[LearningLanguage, str] = {"en": "B2", "de": "A1"}
# Rules for one learning language only; the external-agent guide includes the same files.
LANGUAGE_RULES: dict[LearningLanguage, str] = {"de": "content-rules-de.md"}
RECOGNITION_NOTES: dict[LearningLanguage, str] = {"de": "recognize-de.md"}


def resource(name: str) -> str:
    return PROMPTS.joinpath(name).read_text(encoding="utf-8")


def schema(name: str) -> dict[str, Any]:
    return json.loads(resource(name))


def _fill(text: str, fields: dict[str, str]) -> str:
    return PLACEHOLDER.sub(lambda match: fields[match[1]].strip(), text)


def language_values(work: CoachWork) -> dict[str, str]:
    language = work.learning_language
    meaning = work.native_language if work.translation else language
    rules = resource("content-rules.md")
    if language in LANGUAGE_RULES:
        rules += "\n\n" + resource(LANGUAGE_RULES[language])
    notes = resource(RECOGNITION_NOTES[language]) if language in RECOGNITION_NOTES else ""
    return {
        "learning_language": LANGUAGE_NAMES[language],
        "meaning_language": LANGUAGE_NAMES[meaning],
        "native_language": LANGUAGE_NAMES[work.native_language],
        "level": STATED_LEVELS[language],
        "rules": rules,
        "recognition_notes": notes,
    }


def render(template: str, **values: str) -> str:
    fields = {**values, "card_format": _fill(resource("card-format.md"), values)}
    # One pass keeps placeholder-like text inside learner input or model output literal.
    return _fill(resource(template), fields)
