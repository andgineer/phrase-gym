import hashlib
import json
import os
import struct
import tempfile
from pathlib import Path
from typing import Any, cast

from phrase_gym.coach.bounded_process import OutputLimitError, run_bounded
from phrase_gym.server.languages import LEARNING_LANGUAGES, LearningLanguage

MAX_REFERENCE_BYTES = 8 * 1024 * 1024
MAX_STDERR_BYTES = 64 * 1024
MIN_SAMPLE_RATE = 8_000
MAX_SAMPLE_RATE = 48_000
LOCALES: dict[LearningLanguage, str] = {"en": "en-US", "de": "de-DE"}
RUNTIME_FIELDS = ("executable", "runtime_version")
VOICE_FIELDS = ("model", "model_sha256", "config", "config_sha256")


class MissingVoiceError(RuntimeError):
    """The voice file names no voice for this learning language."""


def _wav(pcm: bytes, sample_rate: int) -> bytes:
    size = len(pcm)
    return b"".join(
        [
            b"RIFF",
            struct.pack("<I", 36 + size),
            b"WAVEfmt ",
            struct.pack("<IHHIIHH", 16, 1, 1, sample_rate, sample_rate * 2, 2, 16),
            b"data",
            struct.pack("<I", size),
            pcm,
        ],
    )


def _strings(decoded: object, required: tuple[str, ...]) -> dict[str, Any]:
    if not isinstance(decoded, dict) or any(
        not isinstance(decoded.get(key), str) for key in required
    ):
        raise TypeError
    return cast(dict[str, Any], decoded)


def _verified_settings(voice: dict[str, Any]) -> dict[str, Any]:
    model = Path(voice["model"])
    config = Path(voice["config"])
    for path, digest in ((model, voice["model_sha256"]), (config, voice["config_sha256"])):
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            raise RuntimeError("Piper model or configuration checksum mismatch")
    return cast(dict[str, Any], json.loads(config.read_text()))


class PiperSpeech:
    def __init__(
        self,
        config_path: Path | None,
        fake: bool = False,
        fake_failures: int = 0,
        timeout: float = 60,
        fake_voices: tuple[LearningLanguage, ...] = LEARNING_LANGUAGES,
    ):
        self.config_path = config_path
        self.fake = fake
        self.fake_failures = fake_failures
        self.timeout = timeout
        self.fake_voices = fake_voices

    def check_voice(self, language: LearningLanguage) -> None:
        """Fails before any preparation work when the language has no usable voice."""
        if self.fake:
            if language not in self.fake_voices:
                raise MissingVoiceError(f"No fake voice for {language}")
            return
        self._configuration(language)

    async def synthesize(self, text: str, language: LearningLanguage) -> bytes:
        if self.fake_failures:
            self.fake_failures -= 1
            raise RuntimeError("Synthetic preparation failure; correct the setup and retry")
        if self.fake:
            self.check_voice(language)
            return _wav(b"\x00\x00" * 2_205, 22_050)
        runtime, voice, settings = self._configuration(language)
        executable = Path(runtime["executable"])
        model = Path(voice["model"])
        config = Path(voice["config"])
        environment = {"PATH": str(executable.parent), "LANG": "C.UTF-8"}
        await self._verify_runtime(executable, str(runtime["runtime_version"]), environment)
        sample_rate = int(settings["audio"]["sample_rate"])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            input_path = root / "input.txt"
            input_path.write_text(text)
            try:
                result = await run_bounded(
                    executable,
                    [
                        "-I",
                        "-m",
                        "piper",
                        "--model",
                        str(model),
                        "--config",
                        str(config),
                        "--input-file",
                        str(input_path),
                        "--output-raw",
                        "--speaker",
                        "0",
                    ],
                    cwd=root,
                    environment=environment,
                    timeout=self.timeout,
                    stdout_limit=MAX_REFERENCE_BYTES - 44,
                    stderr_limit=MAX_STDERR_BYTES,
                )
            except TimeoutError:
                raise RuntimeError("Piper timed out; correct the setup and retry") from None
            except OutputLimitError:
                raise RuntimeError("Piper exceeded a bounded output limit") from None
            if result.returncode or result.stderr:
                raise RuntimeError("Piper failed; no reference audio was accepted")
            if not result.stdout:
                raise RuntimeError("Piper produced no audio")
            return _wav(result.stdout, sample_rate)

    def _configuration(
        self,
        language: LearningLanguage,
    ) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
        if not self.config_path:
            raise RuntimeError(
                "Piper is not configured. Set PHRASE_GYM_PIPER_CONFIG to the voice JSON path.",
            )
        try:
            runtime = _strings(json.loads(self.config_path.read_text()), RUNTIME_FIELDS)
            voices = runtime.get("voices")
            if not isinstance(voices, dict):
                raise TypeError
            if language not in voices:
                raise MissingVoiceError(f"Piper has no voice for {LOCALES[language]}")
            voice = _strings(voices[language], VOICE_FIELDS)
            if not Path(runtime["executable"]).is_file():
                raise RuntimeError("Piper executable does not exist")
            settings = _verified_settings(voice)
            locale = settings.get("language", {}).get("code", "").replace("_", "-")
            sample_rate = int(settings.get("audio", {}).get("sample_rate", 0))
        except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
            raise RuntimeError("Piper voice configuration is malformed") from error
        if locale != LOCALES[language]:
            raise RuntimeError(f"Piper's {language} voice must be {LOCALES[language]}")
        if not MIN_SAMPLE_RATE <= sample_rate <= MAX_SAMPLE_RATE:
            raise RuntimeError("Piper voice has an unsupported sample rate")
        return runtime, voice, settings

    async def _verify_runtime(
        self,
        executable: Path,
        expected: str,
        environment: dict[str, str],
    ) -> None:
        try:
            result = await run_bounded(
                executable,
                [
                    "-I",
                    "-c",
                    "import importlib.metadata as m;print(m.version('piper-tts'))",
                ],
                cwd=executable.parent,
                environment=environment,
                timeout=min(max(self.timeout, 1), 10),
                stdout_limit=256,
                stderr_limit=4_096,
            )
        except (TimeoutError, OutputLimitError) as error:
            raise RuntimeError("Piper runtime identity check failed") from error
        if result.returncode or result.stderr or result.stdout.decode().strip() != expected:
            raise RuntimeError("Piper runtime version does not match its configuration")


def configured_speech() -> PiperSpeech:
    config = os.environ.get("PHRASE_GYM_PIPER_CONFIG")
    return PiperSpeech(
        Path(config) if config else None,
        os.environ.get("PHRASE_GYM_FAKE_TTS") == "1",
    )
