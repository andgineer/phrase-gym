import asyncio
import hashlib
import json
import os
import stat
import sys
from pathlib import Path

import pytest

import phrase_gym.speech as speech_module
from phrase_gym.bounded_process import run_bounded
from phrase_gym.speech import PiperSpeech


def voice_setup(tmp_path: Path, mode: str = "success", version: str = "1.8.0") -> Path:
    executable = tmp_path / "fake-piper"
    executable.write_text(
        f"""#!{sys.executable}
import sys, time
if '-c' in sys.argv:
    print({version!r})
elif {mode!r} == 'timeout':
    time.sleep(5)
elif {mode!r} == 'stderr':
    sys.stderr.write('failure')
elif {mode!r} == 'large-stderr':
    sys.stderr.write('x' * 2048)
elif {mode!r} == 'large':
    sys.stdout.buffer.write(b'x' * 2048)
elif {mode!r} == 'success':
    sys.stdout.buffer.write(b'\\x01\\x00' * 100)
"""
    )
    executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
    model = tmp_path / "voice.onnx"
    config = tmp_path / "voice.onnx.json"
    model.write_bytes(b"model")
    config.write_text(json.dumps({"language": {"code": "en_US"}, "audio": {"sample_rate": 22_050}}))
    voice = tmp_path / "piper-voice.json"
    voice.write_text(
        json.dumps(
            {
                "executable": str(executable),
                "runtime_version": "1.8.0",
                "model": str(model),
                "model_sha256": hashlib.sha256(model.read_bytes()).hexdigest(),
                "config": str(config),
                "config_sha256": hashlib.sha256(config.read_bytes()).hexdigest(),
                "voice": "test",
                "locale": "en-US",
            }
        )
    )
    return voice


def synthesize(path: Path, timeout: float = 1) -> bytes:
    return asyncio.run(PiperSpeech(path, timeout=timeout).synthesize("A complete phrase."))


def test_verified_runtime_produces_bounded_wav(tmp_path: Path) -> None:
    result = synthesize(voice_setup(tmp_path))
    assert result.startswith(b"RIFF")
    assert len(result) == 244


@pytest.mark.parametrize(
    ("mode", "message"),
    [
        ("stderr", "failed"),
        ("empty", "no audio"),
        ("timeout", "timed out"),
    ],
)
def test_child_failures_are_actionable(tmp_path: Path, mode: str, message: str) -> None:
    timeout = 0.05 if mode == "timeout" else 1
    with pytest.raises(RuntimeError, match=message):
        synthesize(voice_setup(tmp_path, mode), timeout=timeout)


@pytest.mark.parametrize(
    ("mode", "limit_name"),
    [("large", "MAX_REFERENCE_BYTES"), ("large-stderr", "MAX_STDERR_BYTES")],
)
def test_output_limits_terminate_child(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, mode: str, limit_name: str
) -> None:
    monkeypatch.setattr(speech_module, limit_name, 128)
    with pytest.raises(RuntimeError, match="bounded output"):
        synthesize(voice_setup(tmp_path, mode))


def test_runtime_version_must_match_configuration(tmp_path: Path) -> None:
    with pytest.raises(RuntimeError, match="version does not match"):
        synthesize(voice_setup(tmp_path, version="9.9.9"))


def test_hash_locale_and_malformed_configuration_fail_before_synthesis(tmp_path: Path) -> None:
    voice_path = voice_setup(tmp_path)
    voice = json.loads(voice_path.read_text())
    voice["model_sha256"] = "0" * 64
    voice_path.write_text(json.dumps(voice))
    with pytest.raises(RuntimeError, match="checksum"):
        synthesize(voice_path)

    voice_path = voice_setup(tmp_path)
    config = Path(json.loads(voice_path.read_text())["config"])
    settings = json.loads(config.read_text())
    settings["language"]["code"] = "de_DE"
    config.write_text(json.dumps(settings))
    voice = json.loads(voice_path.read_text())
    voice["config_sha256"] = hashlib.sha256(config.read_bytes()).hexdigest()
    voice_path.write_text(json.dumps(voice))
    with pytest.raises(RuntimeError, match="English"):
        synthesize(voice_path)

    voice_path.write_text("not json")
    with pytest.raises(RuntimeError, match="malformed"):
        synthesize(voice_path)


def test_cancellation_kills_and_reaps_stubborn_child(tmp_path: Path) -> None:
    executable = tmp_path / "stubborn-child"
    executable.write_text(
        f"""#!{sys.executable}
import pathlib, signal, sys, time
signal.signal(signal.SIGTERM, signal.SIG_IGN)
pathlib.Path(sys.argv[1]).write_text(str(__import__('os').getpid()))
time.sleep(30)
"""
    )
    executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
    pid_path = tmp_path / "child.pid"

    async def scenario() -> int:
        task = asyncio.create_task(
            run_bounded(
                executable,
                [str(pid_path)],
                cwd=tmp_path,
                environment={"PATH": str(tmp_path), "LANG": "C.UTF-8"},
                timeout=60,
                stdout_limit=1_024,
                stderr_limit=1_024,
            )
        )
        for _ in range(100):
            if pid_path.exists():
                break
            await asyncio.sleep(0.01)
        assert pid_path.exists()
        pid = int(pid_path.read_text())
        task.cancel()
        with pytest.raises(asyncio.CancelledError):
            await task
        return pid

    pid = asyncio.run(scenario())
    with pytest.raises(ProcessLookupError):
        os.kill(pid, 0)
