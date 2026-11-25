from typing import Literal

# Prompts name languages in English; the answers are written in the named language.
LANGUAGE_NAMES = {"en": "English"}

Message = Literal["instruction", "unresolved", "no_safe_cue", "too_long"]
MESSAGES: dict[str, dict[Message, str]] = {
    "en": {
        "instruction": "Say the missing wording.",
        "unresolved": "Coach could not confirm one form for this input.",
        "no_safe_cue": "Coach could not make a card that keeps the answer hidden.",
        "too_long": "The text is too long for one card.",
    },
}


def message(key: Message, language: str) -> str:
    return MESSAGES[language][key]
