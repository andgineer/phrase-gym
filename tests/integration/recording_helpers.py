import base64
from uuid import uuid4

from tests.integration.helpers import NOW, wav_bytes


def take(attempt, **changes):
    return {
        "id": str(uuid4()),
        "target_id": attempt["target_id"],
        "revision": attempt["revision"],
        "attempt_id": attempt["id"],
        "captured_at": NOW.isoformat(),
        "mime_type": "audio/wav",
        "duration_ms": 100,
        "complete": True,
        "audio_base64": base64.b64encode(wav_bytes()).decode(),
        **changes,
    }
