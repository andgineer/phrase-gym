import base64
import io
import wave
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
NOW = datetime(2026, 10, 12, 12, tzinfo=timezone.utc)


def publish(client, package):
    staged = client.post("/api/v1/imports", json=package)
    assert staged.status_code == 200, staged.text
    report = staged.json()
    assert report["status"] == "staged", report
    result = client.post(f"/api/v1/imports/{report['id']}/publish", json={"reviewed": True})
    assert result.status_code == 200, result.text
    return result.json()


def phrases(client, lang="en", q=""):
    response = client.get("/api/v1/phrases/search", params={"language": lang, "q": q})
    assert response.status_code == 200
    return response.json()["items"]


def wav_bytes():
    stream = io.BytesIO()
    with wave.open(stream, "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(16000)
        audio.writeframes(b"\x00\x00" * 1600)
    return stream.getvalue()


def upload(client, audio=None, **metadata):
    provenance = {
        "language": "en",
        "text": "How has your day been so far?",
        "role": "reference",
        "origin": "actor",
        "source": "acceptance-test",
        "performer": "Synthetic test fixture",
        "locale": "en-US",
        "preparation_version": "1",
        **metadata,
    }
    return client.post(
        "/api/v1/media",
        json={
            "metadata": provenance,
            "mime_type": "audio/wav",
            "audio_base64": base64.b64encode(wav_bytes() if audio is None else audio).decode(),
        },
    )
