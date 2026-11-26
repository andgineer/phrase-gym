# Set up Coach’s voices

Coach speaks every card's revealed wording with a local Piper voice for that card's learning
language: one English voice and one German voice. The separately running Coach expects
`PHRASE_GYM_PIPER_CONFIG` to name a private JSON file outside the repository. Piper itself is a
separately pinned environment.

On the learner's macOS machine, create that private environment outside every repository with the
same Python minor version as Phrase Gym and these exact packages:

```bash
PIPER="$HOME/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/piper-1.8.0"
python3.13 -m venv "$PIPER"
"$PIPER/bin/python" -I -m pip install --no-deps --only-binary=:all: \
  piper-tts==1.8.0 onnxruntime==1.31.0 flatbuffers==25.12.19 numpy==2.5.4 \
  packaging==25.0 pathvalidate==3.3.1 protobuf==7.36.2
"$PIPER/bin/python" -I -m pip check
```

Download each voice's model and adjacent JSON configuration outside the repository. The German
voice is `de_DE-thorsten-medium` from Piper's voice collection at revision `v1.0.0`:

```bash
VOICES="$HOME/Library/Caches/io.github.andgineer.Phrase-Gym-Coach/voices"
SOURCE=https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/de/de_DE/thorsten/medium
mkdir -p "$VOICES/thorsten-medium"
curl -fL -o "$VOICES/thorsten-medium/de_DE-thorsten-medium.onnx" "$SOURCE/de_DE-thorsten-medium.onnx"
curl -fL -o "$VOICES/thorsten-medium/de_DE-thorsten-medium.onnx.json" \
  "$SOURCE/de_DE-thorsten-medium.onnx.json"
shasum -a 256 "$VOICES"/thorsten-medium/*
```

The German files must have these SHA-256 values:

| File | SHA-256 |
|---|---|
| `de_DE-thorsten-medium.onnx` | `7e64762d8e5118bb578f2eea6207e1a35a8e0c30595010b666f983fc87bb7819` |
| `de_DE-thorsten-medium.onnx.json` | `974adee790533adb273a1ac88f49027d2a1b8f0f2cf4905954a4791e79264e85` |

Calculate the SHA-256 values of the English voice you selected in the same way, then create the
private voice file with one entry per learning language:

```json
{
  "executable": "/absolute/path/to/piper/bin/python",
  "runtime_version": "1.8.0",
  "voices": {
    "en": {
      "model": "/absolute/path/to/en_US-ljspeech-medium.onnx",
      "model_sha256": "64-lowercase-hex-digits",
      "config": "/absolute/path/to/en_US-ljspeech-medium.onnx.json",
      "config_sha256": "64-lowercase-hex-digits",
      "voice": "en_US-ljspeech-medium",
      "locale": "en-US"
    },
    "de": {
      "model": "/absolute/path/to/de_DE-thorsten-medium.onnx",
      "model_sha256": "7e64762d8e5118bb578f2eea6207e1a35a8e0c30595010b666f983fc87bb7819",
      "config": "/absolute/path/to/de_DE-thorsten-medium.onnx.json",
      "config_sha256": "974adee790533adb273a1ac88f49027d2a1b8f0f2cf4905954a4791e79264e85",
      "voice": "de_DE-thorsten-medium",
      "locale": "de-DE"
    }
  }
}
```

Coach verifies the exact installed `piper-tts` version against `runtime_version`, each voice's
model/config hashes, and that the voice's locale matches its learning language: `en-US` for
English and `de-DE` for German. Without a voice for a card's language, Coach reports in the Coach
inbox that it has no voice for that language yet, before any model call; **Retry** prepares the
card once the voice is added, and the other language keeps working. Coach streams the selected
interpreter's output under hard 8 MiB audio, 64 KiB diagnostic, and 60-second limits, terminates
a process that exceeds a bound, and sends WAV references to the Phrase Gym API for atomic
publication. Automated tests use a silent fake child and never invoke real Piper.
