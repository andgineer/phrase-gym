import pytest
from pydantic import ValidationError

from phrase_gym.preparation.speech import SpeechSettings, SpeechVoice, serbian_cyrillic
from phrase_gym.preparation.providers import BrokerProvider, ProviderSettings
from phrase_gym.errors import DomainError


def test_serbian_conversion_digraphs_punctuation_and_display():
    display = "Vidimo se sledeće nedelje. Ljubičasta haljina i džem."
    speech = serbian_cyrillic(display)
    assert speech == "Видимо се следеће недеље. Љубичаста хаљина и џем."
    assert display.startswith("Vidimo")
    assert serbian_cyrillic("Надам се да ћеш доћи на време.") == "Надам се да ћеш доћи на време."


def test_wrong_language_accent_and_lower_sorbian_rejected(tmp_path):
    with pytest.raises(ValidationError, match="phrase language"):
        SpeechSettings.model_validate(
            {"voices": {"sr": [{"engine": "edge", "voice": "en-US-AriaNeural", "locale": "en-US"}]}}
        )
    with pytest.raises(ValidationError, match="Lower Sorbian"):
        SpeechVoice(engine="piper", voice="serbski_institut", locale="sr-RS")
    with pytest.raises(ValidationError, match="locale/accent"):
        SpeechSettings.model_validate(
            {
                "voices": {
                    "en": [
                        {"engine": "edge", "voice": "en-US-AriaNeural", "locale": "en-US"},
                        {"engine": "edge", "voice": "en-GB-SoniaNeural", "locale": "en-GB"},
                    ]
                }
            }
        )
    with pytest.raises(DomainError, match="deliberate"):
        BrokerProvider(ProviderSettings(direct_model="gpt", allow_paid=False), tmp_path)
