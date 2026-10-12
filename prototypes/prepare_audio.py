"""Prepare disposable English reference clips with an installed macOS voice."""

import json
import shutil
import subprocess
import tempfile
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parent
MIN_AUDIO_BYTES = 1000


def main() -> None:
    """Generate whole-phrase assets; never replace imported actor audio."""
    say = shutil.which("say")
    ffmpeg = shutil.which("ffmpeg")
    if not say or not ffmpeg:
        raise SystemExit(
            "Install/configure a preparation provider or import reference audio in the probe.",
        )
    pack = json.loads((ROOT / "drill-flow/fixtures.json").read_text())
    (ROOT / "media").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="phrase-gym-audio-") as folder:
        for target in pack["targets"]:
            if target["language"] != "en":
                continue
            output = ROOT / "media" / (target["id"] + ".wav")
            if output.exists() and output.stat().st_size > MIN_AUDIO_BYTES:
                continue
            temporary = Path(folder) / "speech.aiff"
            subprocess.run(  # noqa: S603 - trusted fixture text; argv, never a shell command.
                [say, "-v", "Samantha", "-r", "155", "-o", str(temporary), target["text"]],
                check=True,
            )
            subprocess.run(  # noqa: S603 - executable resolved locally; argv paths only.
                [
                    ffmpeg,
                    "-loglevel",
                    "error",
                    "-y",
                    "-i",
                    str(temporary),
                    "-ar",
                    "24000",
                    "-ac",
                    "1",
                    str(output),
                ],
                check=True,
            )
            with wave.open(str(output)) as audio:
                if not audio.getnframes():
                    output.unlink()
                    raise SystemExit(
                        "Speech provider returned no audio; reference preparation failed.",
                    )
            print(output.name)


if __name__ == "__main__":
    main()
