# Importing prepared English phrases

Phrase Gym accepts one UTF-8 JSON object. Import format `phrase-gym.package` version `2` is the
current package contract; earlier versions are rejected. The application makes no model or
course-provider call during import; source links are attribution only and are never fetched.

## Schema

| Field | Requirement |
|---|---|
| `format` | Exactly `phrase-gym.package` |
| `version` | Exactly `2`; other versions fail clearly |
| `package_id` | Stable ID for this exact package content; letters, digits, `.`, `_`, `-` |
| `source_id` | Stable identity for the course/unit/source |
| `source_label` | Short learner-readable attribution |
| `source_link` | Optional absolute HTTP(S) attribution URL |
| `learning_language` | `en` |
| `card_language` | `en` |
| `help_language` | `en` |
| `lessons` | 1–20 lessons with at most 50 phrases in total; an ordinary batch has 5–10 phrases |

All reference audio for one package, whether Coach prepares it or an agent supplies it, must fit
one 9 MiB budget: about 3.5 minutes of speech at 22,050 Hz. Fifty long sentences can exceed it;
the import then fails with an error asking you to split the package, so keep packages short.

Each lesson has a stable `lesson_id`, a learner-readable `title` and one or more `phrases`.
One package may contain several lessons; every phrase belongs to exactly one of them.

Each phrase requires:

| Field | Requirement |
|---|---|
| `item_id` | Stable ID, unique within the package |
| `text` | The fixed complete English wording |
| `explanation` | Short English meaning/use help shown after the answer |
| `cues` | One or two distinct situations; each has `situation` and an optional `partner_line` |
| `recall_hints` | Two to six distinct hand-prepared hints of one or two words, at most 40 characters each, never the whole phrase |
| `usage_contexts` | One to six distinct short searchable uses, such as `small talk` or `greetings` |
| `accepted_variants` | Optional genuinely suitable alternatives for the same intention |
| `detailed_explanation` | Optional longer native-language help shown on request |

Unknown fields are rejected, and validation names the lesson, phrase and field to correct.

`source_id` plus `lesson_id` is permanent lesson identity: a later package can add phrases to
that lesson only with the same title. `source_id` plus `item_id` is permanent phrase identity.
Repeating it while that phrase is in the Library is rejected without overwriting it. To replace
a defective phrase, delete it in the Library or its details after confirmation, then import the
corrected phrase in a new package; it becomes a new phrase with fresh reference audio.

The generated machine-readable request schema is available at
[`frontend/openapi.json`](https://github.com/andgineer/phrase-gym/blob/main/frontend/openapi.json).
A complete three-lesson file is
[`examples/meeting-phrases.json`](https://github.com/andgineer/phrase-gym/blob/main/examples/meeting-phrases.json).

## Instruction for an external preparation agent

> Prepare 5–10 useful fixed English phrases from the supplied course material as one Phrase Gym
> package version 2. Preserve the course wording, group phrases into the course's lessons, and
> give the package, source, every lesson and every phrase a stable searchable ID. For each phrase
> provide the complete English `text`, a short plain-English meaning/use explanation, one or two
> distinct short situations that communicate the intention without revealing the target wording,
> two or more hand-picked recall hints of one or two words that help retrieve this exact wording
> (not simply its first or last word), and short `usage_contexts` a learner might search for.
> Add a preceding partner line only when natural. Include only genuinely acceptable variants.
> Use the "So far, so good." phrase in the example file as the model. Do not add raw words,
> variable slots, dialogue graphs, invented quotations, audio URLs, or fields outside the
> documented schema. Return JSON only, then inspect every phrase for intended meaning, natural
> cues, answer leakage, useful hints and contexts, and suitable alternatives before giving it to
> the learner.

Phrase Gym validates structure, not teaching quality. The learner or a semantic reviewer still
checks real prepared content separately.

## Piper voice

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

## API handoff

`POST /api/imports` accepts `{ "package": <version-2 package> }`. The server validates and
stores a text-only request before returning `pending`. Start `phrase-gym coach` separately to
prepare it. `GET /api/imports` and `GET /api/imports/{package_id}` report `pending`, `preparing`,
`added`, or `error`. Retry an error or interrupted preparation with `POST /api/imports/{package_id}/retry`; retrying the
same package content returns its existing status. Coach retrieves work and returns a complete
result through the API; it never opens the application database.

An external agent that already has audio can call `POST /api/imports` with `package` and
`references`, a map from each `item_id` to base64-encoded mono 16-bit PCM WAV bytes. Every
phrase must have one complete playable reference, at most 8 MiB, and all references together must
fit a 9 MiB audio budget. These limits keep the base64 JSON below the API's 16 MiB request limit.
This publishes the batch
atomically and returns `added` without Coach. A malformed or missing reference identifies the
item and rejects the whole request. Retrying identical package and audio returns the original
result; changed content under the same `package_id` is rejected. The browser paste/file import
accepts text-only packages and shows pending Coach work while it is offline.
