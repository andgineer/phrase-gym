import base64
import binascii
import io
import wave

from phrase_gym.server.import_contracts import PhrasePackage

MAX_REFERENCE_BYTES = 8 * 1024 * 1024
MAX_BATCH_REFERENCE_BYTES = 9 * 1024 * 1024
MIN_WAV_BYTES = 44
PCM_SAMPLE_WIDTH = 2
MIN_SAMPLE_RATE = 8_000
MAX_SAMPLE_RATE = 48_000


class ImportConflictError(ValueError):
    pass


def decode_references(package: PhrasePackage, audio: dict[str, str]) -> dict[str, bytes]:
    expected = {phrase.item_id for _, phrase in package.phrases()}
    if set(audio) != expected:
        missing = sorted(expected - set(audio))
        extra = sorted(set(audio) - expected)
        raise ImportConflictError(
            f"Reference items do not match: missing={missing}, extra={extra}",
        )
    decoded: dict[str, bytes] = {}
    total = 0
    for item, value in audio.items():
        decoded[item] = _decode_reference(item, value)
        total += len(decoded[item])
        if total > MAX_BATCH_REFERENCE_BYTES:
            raise ImportConflictError(
                f"phrases.{item}: references exceed the 9 MiB batch audio budget",
            )
    return decoded


def _decode_reference(item: str, value: str) -> bytes:
    try:
        raw = base64.b64decode(value, validate=True)
        if not MIN_WAV_BYTES < len(raw) <= MAX_REFERENCE_BYTES:
            raise ValueError("reference size is invalid")
        with wave.open(io.BytesIO(raw)) as wav:
            if (
                wav.getnchannels() != 1
                or wav.getsampwidth() != PCM_SAMPLE_WIDTH
                or wav.getnframes() == 0
            ):
                raise ValueError("reference must be nonempty mono 16-bit WAV")
            if not MIN_SAMPLE_RATE <= wav.getframerate() <= MAX_SAMPLE_RATE:
                raise ValueError("reference sample rate is invalid")
            if len(wav.readframes(wav.getnframes())) != wav.getnframes() * PCM_SAMPLE_WIDTH:
                raise ValueError("reference audio is incomplete")
        return raw
    except (ValueError, wave.Error, binascii.Error, EOFError) as error:
        raise ImportConflictError(f"phrases.{item}: invalid reference audio: {error}") from error
