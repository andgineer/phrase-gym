import asyncio
import hashlib
import json
import os
import struct
import tempfile
from pathlib import Path

MAX_REFERENCE_BYTES = 12 * 1024 * 1024


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
        ]
    )


class PiperSpeech:
    def __init__(
        self,
        config_path: Path | None,
        fake: bool = False,
        fake_failures: int = 0,
        fake_failure_text: str | None = None,
    ):
        self.config_path = config_path
        self.fake = fake
        self.fake_failures = fake_failures
        self.fake_failure_text = fake_failure_text

    async def synthesize(self, text: str) -> bytes:
        if self.fake_failures and (
            self.fake_failure_text is None or text == self.fake_failure_text
        ):
            self.fake_failures -= 1
            raise RuntimeError("Synthetic preparation failure; correct the setup and retry")
        if self.fake:
            return _wav(b"\x00\x00" * 2_205, 22_050)
        if not self.config_path:
            raise RuntimeError(
                "Piper is not configured. Set PHRASE_GYM_PIPER_CONFIG to the voice JSON path."
            )
        voice = json.loads(self.config_path.read_text())
        executable = Path(voice["executable"])
        model = Path(voice["model"])
        config = Path(voice["config"])
        for path, digest in (
            (model, voice["model_sha256"]),
            (config, voice["config_sha256"]),
        ):
            if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
                raise RuntimeError("Piper model or configuration checksum mismatch")
        settings = json.loads(config.read_text())
        if settings.get("language", {}).get("code", "").replace("_", "-") != "en-US":
            raise RuntimeError("Piper voice must be English (en-US)")
        sample_rate = int(settings.get("audio", {}).get("sample_rate", 0))
        if not 8_000 <= sample_rate <= 48_000:
            raise RuntimeError("Piper voice has an unsupported sample rate")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            input_path = root / "input.txt"
            output_path = root / "reference.pcm"
            input_path.write_text(text)
            process = await asyncio.create_subprocess_exec(
                str(executable),
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
                cwd=root,
                env={"PATH": str(executable.parent), "LANG": "C.UTF-8"},
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            try:
                stdout, stderr = await asyncio.wait_for(process.communicate(), 60)
            except TimeoutError:
                process.kill()
                await process.wait()
                raise RuntimeError("Piper timed out; correct the setup and retry") from None
            if process.returncode or stderr or len(stdout) > MAX_REFERENCE_BYTES:
                raise RuntimeError("Piper failed or exceeded the reference audio limit")
            output_path.write_bytes(stdout)
            if not stdout:
                raise RuntimeError("Piper produced no audio")
            return _wav(stdout, sample_rate)


def configured_speech() -> PiperSpeech:
    config = os.environ.get("PHRASE_GYM_PIPER_CONFIG")
    return PiperSpeech(
        Path(config) if config else None,
        os.environ.get("PHRASE_GYM_FAKE_TTS") == "1",
        int(os.environ.get("PHRASE_GYM_FAKE_TTS_FAILURES", "0")),
        os.environ.get("PHRASE_GYM_FAKE_TTS_FAIL_TEXT"),
    )
