import hashlib
import json
import os
import struct
import tempfile
from pathlib import Path
from typing import Any, cast

from phrase_gym.coach.bounded_process import OutputLimitError, run_bounded

MAX_REFERENCE_BYTES = 12 * 1024 * 1024
MAX_STDERR_BYTES = 64 * 1024
MIN_SAMPLE_RATE = 8_000
MAX_SAMPLE_RATE = 48_000


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


class PiperSpeech:
    def __init__(
        self,
        config_path: Path | None,
        fake: bool = False,
        fake_failures: int = 0,
        fake_failure_text: str | None = None,
        timeout: float = 60,
    ):
        self.config_path = config_path
        self.fake = fake
        self.fake_failures = fake_failures
        self.fake_failure_text = fake_failure_text
        self.timeout = timeout

    async def synthesize(self, text: str) -> bytes:
        if self.fake_failures and (
            self.fake_failure_text is None or text == self.fake_failure_text
        ):
            self.fake_failures -= 1
            raise RuntimeError("Synthetic preparation failure; correct the setup and retry")
        if self.fake:
            return _wav(b"\x00\x00" * 2_205, 22_050)
        voice, settings = self._configuration()
        executable = Path(voice["executable"])
        model = Path(voice["model"])
        config = Path(voice["config"])
        environment = {"PATH": str(executable.parent), "LANG": "C.UTF-8"}
        await self._verify_runtime(executable, str(voice["runtime_version"]), environment)
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

    def _configuration(self) -> tuple[dict[str, Any], dict[str, Any]]:
        if not self.config_path:
            raise RuntimeError(
                "Piper is not configured. Set PHRASE_GYM_PIPER_CONFIG to the voice JSON path.",
            )
        try:
            decoded_voice: object = json.loads(self.config_path.read_text())
            required = (
                "executable",
                "runtime_version",
                "model",
                "model_sha256",
                "config",
                "config_sha256",
            )
            if not isinstance(decoded_voice, dict) or any(
                not isinstance(decoded_voice.get(key), str) for key in required
            ):
                raise ValueError
            voice = cast(dict[str, Any], decoded_voice)
            executable = Path(voice["executable"])
            model = Path(voice["model"])
            config = Path(voice["config"])
            if not executable.is_file():
                raise RuntimeError("Piper executable does not exist")
            for path, digest in (
                (model, voice["model_sha256"]),
                (config, voice["config_sha256"]),
            ):
                if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
                    raise RuntimeError("Piper model or configuration checksum mismatch")
            settings = cast(dict[str, Any], json.loads(config.read_text()))
            locale = settings.get("language", {}).get("code", "").replace("_", "-")
            sample_rate = int(settings.get("audio", {}).get("sample_rate", 0))
        except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
            raise RuntimeError("Piper voice configuration is malformed") from error
        if locale != "en-US":
            raise RuntimeError("Piper voice must be English (en-US)")
        if not MIN_SAMPLE_RATE <= sample_rate <= MAX_SAMPLE_RATE:
            raise RuntimeError("Piper voice has an unsupported sample rate")
        return voice, settings

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
        int(os.environ.get("PHRASE_GYM_FAKE_TTS_FAILURES", "0")),
        os.environ.get("PHRASE_GYM_FAKE_TTS_FAIL_TEXT"),
    )
