# Set up Coach’s English voice

The separately running Coach expects `PHRASE_GYM_PIPER_CONFIG` to name a private JSON file outside the
repository. Piper itself is a separately pinned environment.

On the learner's macOS machine, create that private environment outside every repository with the
same Python minor version as Phrase Gym and these exact packages:

```bash
PIPER="$HOME/Library/Application Support/io.github.andgineer.Phrase-Gym/piper-1.8.0"
python3.13 -m venv "$PIPER"
"$PIPER/bin/python" -I -m pip install --no-deps --only-binary=:all: \
  piper-tts==1.8.0 onnxruntime==1.31.0 flatbuffers==25.12.19 numpy==2.5.4 \
  packaging==25.0 pathvalidate==3.3.1 protobuf==7.36.2
"$PIPER/bin/python" -I -m pip check
```

Download the selected model and adjacent JSON configuration outside the repository, calculate both
SHA-256 values, and create the private voice file with this shape:

```json
{
  "executable": "/absolute/path/to/piper/bin/python",
  "runtime_version": "1.8.0",
  "model": "/absolute/path/to/en_US-ljspeech-medium.onnx",
  "model_sha256": "64-lowercase-hex-digits",
  "config": "/absolute/path/to/en_US-ljspeech-medium.onnx.json",
  "config_sha256": "64-lowercase-hex-digits",
  "voice": "en_US-ljspeech-medium",
  "locale": "en-US"
}
```

Coach verifies the exact installed `piper-tts` version against `runtime_version`, model/config
hashes, and English locale. It streams the selected interpreter's output under hard 8 MiB audio,
64 KiB diagnostic, and 60-second limits, terminates a process that exceeds a bound, and sends WAV references to the Phrase Gym API for atomic publication. Automated tests use a silent fake child and never invoke
real Piper.
