# Importing prepared English phrases

Phrase Gym accepts one UTF-8 JSON object. Import format `phrase-gym.package` version `1` is the
stable stage-1 package contract. The application makes no model or course-provider call during
import; source links are attribution only and are never fetched.

## Schema

| Field | Requirement |
|---|---|
| `format` | Exactly `phrase-gym.package` |
| `version` | Exactly `1`; unknown versions fail clearly |
| `package_id` | Stable ID for this exact package content; letters, digits, `.`, `_`, `-` |
| `source_id` | Stable identity for the course/unit/source |
| `source_label` | Short learner-readable attribution |
| `source_link` | Optional absolute HTTP(S) attribution URL |
| `learning_language` | `en` |
| `card_language` | `en` |
| `help_language` | `en` |
| `cards` | 1–20 cards; an ordinary batch should contain 5–10 |

Each card requires a stable `item_id`, a fixed complete English `answer`, a short English
`explanation`, and one or two `cues`. A cue requires a natural `situation` and may include a
`partner_line`. Optional `accepted_variants` must be genuinely suitable for the same intention;
`detailed_explanation` may add longer help. Unknown fields are rejected.

`source_id` plus `item_id` is permanent card identity. Repeating that identity in another package
is rejected without overwriting it. Correction/replacement with suspension belongs to the later
library stage.

The generated machine-readable request schema is available at
[`frontend/openapi.json`](https://github.com/andgineer/phrase-gym/blob/main/frontend/openapi.json).
A complete five-card file is
[`examples/meeting-phrases.json`](https://github.com/andgineer/phrase-gym/blob/main/examples/meeting-phrases.json).

## Instruction for an external preparation agent

> Prepare 5–10 useful fixed English phrases from the supplied course material as one Phrase Gym
> package version 1. Preserve the course wording and give the package, source, and every item a
> stable searchable ID. For each card provide the complete English phrase, a short plain-English
> meaning/use explanation, and one or two distinct natural situations that communicate the
> intention without revealing the target wording. Add a preceding partner line only when natural.
> Include only genuinely acceptable variants. Do not add raw words, variable slots, dialogue
> graphs, invented quotations, audio URLs, or fields outside the documented schema. Return JSON
> only, then inspect every card for intended meaning, natural cues, answer leakage, and suitable
> alternatives before giving it to the learner.

Phrase Gym validates structure, not teaching quality. The learner or a semantic reviewer still
checks real prepared content separately.

## Piper voice

The application expects `PHRASE_GYM_PIPER_CONFIG` to name a private JSON file outside the
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

Phrase Gym verifies the exact installed `piper-tts` version against `runtime_version`, model/config
hashes, and English locale. It streams the selected interpreter's output under hard 12 MiB audio,
64 KiB diagnostic, and 60-second limits, terminates a process that exceeds a bound, and saves WAV
references before publishing the package. Automated tests use a silent fake child and never invoke
real Piper.
