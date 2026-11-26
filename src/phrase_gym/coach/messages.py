from typing import Literal

Message = Literal["instruction", "whole_instruction", "no_safe_cue", "too_long", "no_voice"]
MESSAGES: dict[str, dict[Message, str]] = {
    "en": {
        "instruction": "Say the missing wording.",
        "whole_instruction": "Say the whole word or phrase.",
        "no_safe_cue": "Coach could not make a card that keeps the answer hidden.",
        "too_long": "The text is too long for one card.",
        "no_voice": "Coach has no reference voice for this language yet. Retry once it has one.",
    },
    "ru": {
        "instruction": "Произнесите пропущенное.",
        "whole_instruction": "Произнесите слово или фразу целиком.",
        "no_safe_cue": "Coach не смог составить карточку, в которой ответ остаётся скрытым.",
        "too_long": "Текст слишком длинный для одной карточки.",
        "no_voice": "У Coach пока нет голоса для этого языка. Повторите, когда он появится.",
    },
}


def message(key: Message, language: str) -> str:
    return MESSAGES[language][key]
