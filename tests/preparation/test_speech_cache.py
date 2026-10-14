import asyncio
from pathlib import Path

from phrase_gym.preparation.speech import SpeechAdapter, SpeechSettings


def test_whole_phrase_cache_identity_voice_text_and_serbian_input(prepared_runtime, monkeypatch):
    _, media, _, _ = prepared_runtime
    calls = []

    class FakeEdge:
        def __init__(self, text, voice):
            calls.append((text, voice))

        async def save(self, path):
            Path(path).write_bytes(Path("tests/fixtures/media/tone.mp3").read_bytes())

    monkeypatch.setattr("phrase_gym.preparation.speech.edge_tts.Communicate", FakeEdge)
    configured = SpeechSettings.model_validate(
        {
            "voices": {
                "sr": [
                    {"engine": "edge", "voice": "sr-RS-SophieNeural", "locale": "sr-RS"},
                ]
            }
        }
    )
    adapter = SpeechAdapter(media, configured)
    text = "Vidimo se sledeće nedelje."
    first = asyncio.run(adapter.prepare(text, "sr", "fixture"))
    assert asyncio.run(adapter.prepare(text, "sr", "fixture")) == first
    assert calls == [("Видимо се следеће недеље.", "sr-RS-SophieNeural")]
    asset = media.get(first)
    assert asset.metadata.text == text and asset.metadata.speech_text.startswith("Видимо")
    assert asset.available
    configured.voices["sr"][0].voice = "sr-RS-NicholasNeural"
    assert asyncio.run(adapter.prepare(text, "sr", "fixture")) != first
    assert asyncio.run(adapter.prepare("Dobro jutro.", "sr", "fixture")) != first
