import io

import av
from av.error import FFmpegError

from phrase_gym.errors import DomainError


def validate_take(audio: bytes, mime_type: str) -> None:
    try:
        with av.open(io.BytesIO(audio), mode="r") as container:
            expected = {
                "audio/wav": "wav",
                "audio/mp4": "mov",
                "audio/webm": "webm",
                "audio/ogg": "ogg",
                "audio/mpeg": "mp3",
            }[mime_type.split(";")[0].lower()]
            if expected not in container.format.name.split(","):
                raise DomainError("Recording MIME type does not match its container", 422)
            if not container.streams.audio or not sum(1 for _frame in container.decode(audio=0)):
                raise DomainError("A complete take must contain decodable audio", 422)
    except (FFmpegError, ValueError) as error:
        raise DomainError("Complete recording audio could not be decoded", 422) from error
