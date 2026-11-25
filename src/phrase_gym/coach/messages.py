from typing import Literal

# Prompts name languages in English; the answers are written in the named language.
LANGUAGE_NAMES = {"en": "English", "ru": "Russian"}

Message = Literal["instruction", "no_safe_cue", "too_long", "english_only"]
MESSAGES: dict[str, dict[Message, str]] = {
    "en": {
        "instruction": "Say the missing wording.",
        "no_safe_cue": "Coach could not make a card that keeps the answer hidden.",
        "too_long": "The text is too long for one card.",
        "english_only": "Coach prepares cards with examples for English only.",
    },
    "ru": {
        "instruction": "Произнесите пропущенное.",
        "no_safe_cue": "Coach не смог составить карточку, в которой ответ остаётся скрытым.",
        "too_long": "Текст слишком длинный для одной карточки.",
        "english_only": "Coach готовит карточки с примерами только для английского.",
    },
}


def message(key: Message, language: str) -> str:
    return MESSAGES[language][key]
