# Phrase Gym Coach desktop

These instructions describe the existing Rust prototype, not the target architecture. They retain working commands, reusable Piper setup and measurement evidence. The [delivery ladder](../specs/plan/implementation-plan.md) replaces the native Coach with a local Python application and the retained Vue GUI.

Coach currently prepares content through Server's durable inbox. The prototype accepts up to five English items per batch, including fixed phrases, reusable frames and contextual words.

## Build, run and smoke-test

Use Node from `.nvmrc`, then:

```bash
cargo xtask app-build
cargo xtask coach-bundle
```

Automated browser checks mute every context while retaining native decoding, playback clocks and media events. Their silence is intentional and does not establish listening quality. Live Reference playback, recording and microphone permission paths require attended learner interaction; do not trigger them during unattended verification.

The unsigned macOS bundle is `target/coach-bundle/Phrase Gym Coach.app`. `cargo xtask coach-smoke` launches the packaged smoke checks; these native checks require attended authorization. Never trigger native dialogs or login automatically. Windows builds retain their platform tests; actual Windows CLI qualification remains pending.

## Local English lesson demo

Run the matching Server with the built client and the existing standard Server data directory. If a Server already owns the port, stop it gracefully before starting the matching build; preserve its database and media unless the build changes the schema, profile or card shape. In demo mode such a change resets the demo data: move the `demo-server` folder aside, together with Coach's `text-receipt.json` and `pending-text.json` that refer to it, then refill through ordinary intake. Do not create an isolated demo data copy or import a prepared lesson:

```bash
target/release/phrase-gym-server serve --data-dir "$HOME/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/demo-server" --client-dir frontend/dist --port 18487
```

Follow the [standing local English demo authorization](../specs/plan/implementation-plan.md#standing-authorization-for-the-local-english-demo). Reuse the existing Codex subscription login, repository workspace and installed local Piper configuration. Routine preparation needs no new budget, workspace or login. Launch `target/coach-bundle/Phrase Gym Coach.app` from Finder.

Paste one to five English items into **English material**, one per line, type them and press **Command-Return** (Ctrl+Enter on Windows), or drop one plain UTF-8 text file (up to 4,000 bytes total, 800 bytes per item). The caption under the box lists these gestures. Any plain text file qualifies, whatever its name or extension (`.txt`, `.text`, `.md` or none); rich-text, binary, non-UTF-8 files, folders and aliases are refused with a specific reason. Use ordinary words and complete phrases. Coach infers a defensible word sense and reusable construction from the text, available context and pinned preferences; it prepares prompts, both help layers and speech. This deliberate gesture immediately starts automatic durable intake, preparation, reference synthesis and atomic publication. Typing alone and opening/closing Settings never submit. Material pasted or dropped while Coach is still taking in earlier input is refused with that explanation; paste or drop it again once Coach shows it is preparing. Original material is retained in the inbox context; each item keeps its source identity. A new lesson is named after its items, for example `impediment` or `Let's get started. · Could you repeat that, please? · …`. After publication the main window names what was added (`Added: impediment`). Supplied valid reference audio remains available through the API and is never silently replaced.

The main surface contains the paste/drop area, concise progress/confirmed additions and scoped English library counts. Counts describe logical targets, not word contexts; stale/offline counts never appear as zero. **Phrase Gym Coach → Settings…** or **Cmd-,** opens a separate window for the preparation tool (Codex or Claude, saved for the next preparation), appearance, Server connection, recovery and diagnostics. Settings uses the saved local Server URL even when it is offline. The current eframe 0.37 secondary-window AccessKit integration does not expose Settings controls to native accessibility clients; actual VoiceOver acceptance remains open. Controls remain keyboard reachable, without claiming attended acceptance.

After confirmed additions, open browser practice and choose **Start session**. Every new card has a short explanation in the help language, shown with the revealed answer and in first study, and a detailed Russian explanation (meaning, translation, usage and, for a word, its sense, examples and forms) that appears only after **Show & explain**. Practice never shows lesson names during a session; a revealed card offers **Open lesson** instead. Lists show a word by the word and its sense; its details page leads with the word, lists every example sentence with its reference and offers the same **Show & explain**. Start studies at most five new targets, ungraded, then **Next** completes each presentation. Frames show the fixed pattern and permitted slot examples after reveal. Word prompts rotate short gapped contexts; their revealed whole-sentence answers and exact references stay paired. Say the whole sentence and rate the selected word/sense rather than unrelated slips. Word contexts share one target and schedule. A later eligible session hides the answer until reveal; ratings refer to the original silent or recorded response. Eligible recall creates real due history. Short sessions end without forcing a wait. Published material/references/takes and unfinished session bindings survive reload and Server restart and remain usable with Coach closed.

In the browser, the global **+** opens a clipboard preview over the current screen. Denied, empty or unavailable clipboard access leaves an editable box. Review the items, optionally add a note, and press **Send**; nothing from the preview is transmitted before Send. The original card and any retained capture stay bound to their initiating attempt. Clipboard reads occur only in the + gesture and late results cannot replace edits or reopen a closed preview.

**Practice now** appears when Due and New are zero and material is ready, and after a completed round. It starts a voluntary round in the current language and lesson scope, with up to 20 ready targets. An unfinished round resumes instead of being replaced. Ratings and every take are retained, but voluntary ratings never change due times or memory state or count as independent recall, including with help or failure. Genuine first study still uses ungraded Next and keeps later first-check eligibility. If nothing is ready, the client explains that valid cues and reference audio are needed.

One input can be outstanding locally. Browser storage and Coach's `pending-text.json` retain exact raw text and stable submission identity until Server acknowledges durable receipt. A crash or lost receipt retries that same identity; it does not synthesize before acceptance. Server owns accepted generations, leases and publication receipts. Generation-bound, exact-text reference checkpoints retain verified speech so a retry can reuse matching bytes. Dropping or pasting the exact failed material again retries its retained generation; the browser status panel and Coach Settings also offer explicit preparation retry. `phrase-gym-add-text --retry-preparation "the same phrase" http://127.0.0.1:18487` verifies the retained accepted text and origin before requesting retry. A local Server outage is distinct from publication.

### Bounded Piper speech proof

The English demo candidate is `en_US-ljspeech-medium`, Piper 1.8.0, ONNX Runtime 1.x, one speaker, 22,050 Hz. The [model card](https://huggingface.co/rhasspy/piper-voices/blob/c10ece1aade47bb51c153c893d14e5bf8e5b7117/en/en_US/ljspeech/medium/MODEL_CARD) identifies scratch training on [LJ Speech](https://keithito.com/LJ-Speech-Dataset/), whose author declares the dataset public domain without use restrictions. The voices repository declares MIT in its [README metadata](https://huggingface.co/rhasspy/piper-voices/blob/c10ece1aade47bb51c153c893d14e5bf8e5b7117/README.md). This choice avoids assuming acceptance of Lessac's research-data terms. Primary voice quality still requires attended listening.

Piper is a separately installed speech tool: Phrase Gym neither bundles nor builds it, and Coach only runs it as a child process. Piper 1.8.0 is GPL-3.0-or-later and ONNX Runtime is MIT. Install it into its own Python virtual environment outside any repository with exactly these versions; the learner's macOS runtime uses Homebrew CPython 3.13 and lives in Coach's settings folder:

```bash
PIPER="$HOME/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/piper-1.8.0"
python3.13 -m venv "$PIPER"
"$PIPER/bin/python" -I -m pip install --no-deps --only-binary=:all: \
  piper-tts==1.8.0 onnxruntime==1.31.0 flatbuffers==25.12.19 numpy==2.5.4 \
  packaging==25.0 pathvalidate==3.3.1 protobuf==7.36.2
"$PIPER/bin/python" -I -m pip check
```

The full pinned list with `--no-deps` keeps the runtime reproducible, and `pip check` confirms it is complete. The runtime identity recorded with each reference includes the interpreter's hash and version, so replacing the interpreter changes cache identity.

Download the model and adjacent JSON configuration from that exact Hugging Face revision into an external cache. Configure `piper-voice.json` in Coach's settings directory, using absolute paths:

```json
{
  "executable": "/absolute/path/to/piper-1.8.0/bin/python",
  "runtime_version": "1.8.0",
  "model": "/absolute/path/to/en_US-ljspeech-medium.onnx",
  "model_sha256": "6f52a751e2349abe7a76735eb09dc1875298c77ea2342ffd2fef79ff81b87f22",
  "config": "/absolute/path/to/en_US-ljspeech-medium.onnx.json",
  "config_sha256": "141d612cc0a95ed7efc1ca936b845c2364967f2e9217c5dbfcf69fc4d6c65860",
  "voice": "en_US-ljspeech-medium",
  "locale": "en-US"
}
```

The interpreter uses isolated mode. The speech runner verifies package versions, hashes, locale and speaker count, stages immutable bytes outside repositories, supervises a maximum 60-second process with bounded output, then validates nonempty PCM and returns WAV. Exact text, speech input, voice, runtime identity, model/config hashes and settings accompany generated media. A failed synthesis retains text and publishes nothing. These checks establish byte/provenance conformance, not pronunciation or listening quality. German and Serbian voices remain unaccepted.

To check a runtime with one real synthesis that writes a WAV and plays nothing, put the `piper-voice.json` contents under `voice` and a request such as `{"language": "en", "text": "Let's get started.", "speech_text": "Let's get started.", "source": "runtime-check", "role": "reference"}` under `request` in one JSON file, then run `cargo run --locked --package phrase-gym-coach --example piper_probe -- --authorized-local-synthesis <that file> <absolute staging folder outside repositories>`.

### Measurement preparation

Freeze synthetic development and held-out cases before real provider calls. The [development fixture](../fixtures/coach/working-set.json), the [fourth held-out freeze](../fixtures/coach/working-set-held-out-4.json) (spent, like the [third](../fixtures/coach/working-set-held-out-3.json) and [second](../fixtures/coach/working-set-held-out-2.json)) and the everyday [shopping](../fixtures/coach/everyday-shopping.json) and [visiting](../fixtures/coach/everyday-visiting.json) batches freeze English material and semantic checks before their measurements. Routine local English demo runs use the [standing authorization](../specs/plan/implementation-plan.md#standing-authorization-for-the-local-english-demo) without per-session or per-run budget approval. Existing timeout, output/media, retry and daily-start limits remain engineering guardrails; count failures and inspect each concrete result before retrying. Separate authorization applies to broader unrequested experiments, different providers and paid API execution. No automatic paid API fallback or unbounded retries are permitted.

Keep the full matrix in the implementation plan: fixed phrases, alternatives, slots, words/senses/forms, dialogue/exposure, preferred/confusable/dependency behavior, ambiguity, absent preferences, and supplied/missing/invalid speech across English, German and Serbian. Freeze unseen held-out wording separately. Record exact executable/model/configuration, fixture identity, elapsed time, unknown usage where unavailable, accepted/failed outputs, and separate fresh semantic review and attended listening. A successful English result does not establish the matrix.

## Per-user folders

| Purpose | macOS | Windows |
|---|---|---|
| Settings and instance lock | `~/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach` | `%APPDATA%\andgineer\Phrase Gym Coach\config` |
| Speech cache | `~/Library/Caches/io.github.andgineer.Phrase-Gym-Coach` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\cache` |
| Logs | `~/Library/Logs/Phrase Gym Coach` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\data\logs` |
| Codex login and customizations | normal `~/.codex` | normal `%USERPROFILE%\.codex` |
| Private Codex attempt files | configured repository's ignored `.phrase-gym-coach/attempts/` | same workspace-relative path |
| Private Claude attempt files | Coach cache `attempts/` | Coach cache `attempts\` |

Settings contain no application credentials and Coach never calls an OS credential store. Authentication tokens are neither copied nor included in job files. Repository attempts are ignored; never commit private logs or learner input. Keep Server data, including recordings, outside the permitted repository workspace. The demo reuses the existing per-user `demo-server` directory beside Coach settings. This standard Server storage and ignored repository attempt files are runtime storage, not separate temporary demo workspaces, profiles or authentication setups. A second local instance is refused.

## Agent CLI setup

Use native Codex 0.163.0 (Homebrew cask on macOS). Coach resolves its canonical executable without shell initialization. Node-script CLI installs are unsupported. Reuse the existing login and verify `codex login status` says `Logged in using ChatGPT`. Only when that login is actually absent does first-time setup require the learner to run `codex login`. Coach inspects status without forcing a login-method change before launching a subscription-only attempt. It never launches login or copies credentials. API-key logins fail readiness, and exported API keys/proxies/parent agent sessions are excluded from the child environment.

For Claude, use the native installer build (`~/.local/bin/claude`), version 2.1.309 or newer, signed in with the Claude subscription: `claude auth status` must report `"loggedIn": true` and `"authMethod": "claude.ai"`. Coach checks the version, the presence of every isolation option in `claude --help` and the login before each attempt, without starting a model, and never uses an API key.

At the learner's explicit direction, local Coach uses the normal Codex home and this repository as its working directory. Normal-home customizations and skills may load; automatic repository instruction loading is disabled for preparation with `project_doc_max_bytes=0`; this configuration does not claim isolation from them. `--ignore-user-config` excludes config.toml settings, and `--ignore-rules` excludes execpolicy .rules loading, while the explicit preparation prompt remains authoritative for its task. Claude has its own measured invocation below and is selected in Coach Settings.

## Verified CLI invocation

### Every attempt

The prompt, pinned input/profile/guide and schema live in a private attempt directory; the short bootstrap names the absolute prompt path. Standard input stays closed. Process supervision owns output bounds, cancellation, stream closure, deadlines and descendant cleanup. Exit zero with malformed, empty, truncated or changed-identity output is a failure. Publication requires a current fenced Server lease and verified exact reference bytes.

### Readiness, failures and usage

Codex checks exact native version and ChatGPT status before execution; Claude checks its minimum version, the presence of every isolation option and its claude.ai login. Authentication/quota errors stop preparation until explicit retry. Unparsed Codex token usage stays unknown to the runner; bounded measurements inspect the anchored `tokens used` stderr trailer when available. Claude's result envelope reports token usage, which the private attempt diagnostics retain. Timing and schema validation do not imply semantic acceptance. Private provider output is separate from sanitized status. Server receipts, rather than a CLI success message, confirm publication.

### Codex

The argument builder pins `exec --skip-git-repo-check --ignore-user-config --ignore-rules --ephemeral`, model `gpt-5.6-luna`, low effort, `forced_login_method="chatgpt"`, approval `never`, web search disabled and no persisted history. Automatic project instruction loading is disabled so development orchestration does not steer preparation. Apps, browser/computer/image tools, goals, hooks, plugins, multi-agent execution and image reading are disabled. The environment is allowlisted and `CODEX_HOME` points directly at the normal home.

The staged generated-contract schema is passed through `--output-schema`, with closed objects, required nullable fields and typed frame/word tasks; malformed output is never repaired by the runner.

An explicit permission profile denies filesystem root, permits minimal system/runtime reads and Codex installation reads, denies shared temporary paths, permits the repository workspace with an explicit `.env` denial and disables command network access. Repository cwd intentionally permits repository access; it is no longer an attempt-only workspace. This is a local trusted-user configuration, not a claim that normal-home customizations are isolated. A native OS sandbox check with a synthetic temporary `.env` confirmed that ordinary workspace input is readable while that secret-file rule denies reads; no real `.env` was opened and no model was started for this check. Recheck affected behavior on version/configuration changes.

### Claude

```text
<claude> -p --model sonnet --effort low --safe-mode --restricted --strict-mcp-config
  --tools Read --allowed-tools Read --permission-prompts none --no-session-persistence
  --disable-slash-commands --output-format json --json-schema <staged schema> -- "Read your task from <prompt> and execute it."
env += CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 ENABLE_CLAUDEAI_MCP_SERVERS=false
```

The working directory is a fresh private attempt folder under Coach's cache, never the repository. `--restricted` confines the Read tool to that folder; `--json-schema` makes the CLI validate the answer against the same staged schema Codex receives, and the JSON result envelope carries it as `structured_output` together with token usage, which the private `execution.json` retains. Exit zero is not enough: the envelope must report `success` with a structured object. Never pass `--bare`, which ignores the subscription login.

Version rule: Coach requires Claude Code 2.1.309 or newer and refuses any build whose `--help` no longer lists one of the flags above. The native installer replaces older builds as it updates, so the previous exact pin (2.1.302) had already disappeared from the machine; a minimum plus flag presence keeps the measured guarantees while surviving forward updates. A newer build that changes a flag's behavior without removing it is not detected automatically, so repeat the probe below after a large Claude update.

Measured on 2026-10-23 with Claude Code 2.1.309 (macOS 27, Apple Silicon, Pro subscription, model alias `sonnet` resolving to claude-sonnet-5-5, low effort):

- A synthetic probe with all flags above listed only the `Read` and `StructuredOutput` tools; reading a canary outside the attempt folder was denied with "--restricted confines the file tools to the working directory" and recorded as a permission denial; the structured answer arrived in the envelope. 8.6 s.
- `examples/working_set_measurement.rs` submitted the frozen development batch through the ordinary durable intake of a disposable Server with disposable Coach folders: the CLI took 34.2 s (5 turns, 6,008 output tokens, 349 thinking tokens, 17.5 KB envelope, 13 KB inline schema), and validation, Piper speech and atomic publication of all five targets finished in 47.6 s, within the 90 s attempt limit.

On 2026-10-24 the packaged Coach, with Claude selected in Settings, filled the reset demo Server from browser Send, one run per batch:

| Batch | Claude | CLI time | Turns | Output tokens | Result |
|---|---|---|---|---|---|
| Development | 2.1.309 | 35.3 s | 5 | 5,995 | 5 published |
| Held-out (second freeze) | 2.1.309 | 0.2 s | 0 | 0 | refused: repeated hint word made the schema invalid; fixed |
| Held-out rerun | 2.1.310 | 39.9 s | 6 | 6,508 | 5 published |
| Shopping | 2.1.310 | 22.6 s | 5 | 3,289 | 5 published |
| Visiting | 2.1.310 | 37.6 s | 6 | 6,543 | 5 published |

Claude updated itself from 2.1.309 to 2.1.310 between runs and passed the version and flag checks. Every published card has an English short explanation, Russian details and verified reference speech. The shopping and visiting batches kept their intended reusable construction as a fixed phrase. Semantic review and listening are separate.

Later on 2026-10-24, after the answer-free-situation guide (v3), the demo Server was reset and refilled the same way:

| Batch | Claude | CLI time | Turns | Output tokens | Result |
|---|---|---|---|---|---|
| Development | 2.1.310 | 0.5 s | 1 | 0 | stopped: subscription session limit (429); retried its generation after the reset |
| Development retry | 2.1.310 | 29.2 s | 5 | 4,234 | 5 published |
| Held-out (third freeze) | 2.1.310 | 32.0 s | 5 | 4,455 | 5 published |
| Shopping | 2.1.310 | 32.5 s | 5 | 4,523 | 5 published |
| Visiting | 2.1.310 | 30.0 s | 5 | 4,214 | 5 published |

All 20 targets have both help layers and verified speech; every batch has a reusable frame (8 frames, 8 fixed phrases, 4 words), and no fixed-phrase or frame situation reuses a content word of its answer. Some situations became stilted to avoid answer words, and some frame situations paraphrase the slot value instead of naming it.

On 2026-10-24, after the plain-frame-value guide (v4), the demo Server was reset and refilled once more, and Codex was measured once on a disposable Server:

| Batch | Tool | CLI time | Turns | Output tokens | Result |
|---|---|---|---|---|---|
| Development | Codex 0.163.0, gpt-5.6-luna, low effort | 90.0 s | – | – | timed out with no output; no retry |
| Development | Claude 2.1.310 | 30.6 s | 5 | 4,235 | 5 published |
| Held-out (fourth freeze) | Claude 2.1.310 | 32.3 s | 5 | 4,235 | 5 published |
| Shopping | Claude 2.1.310 | 34.2 s | 5 | 4,742 | 5 published |
| Visiting | Claude 2.1.310 | 45.5 s | 5 | 7,447 | 5 published |

All 20 Claude targets have both help layers and verified speech (7 frames, 9 fixed phrases, 4 words); frame situations name their values plainly and no situation reuses a content word of its own answer. One word cue ("Explain them better." for clarify) contains a word that fits the gap, and one situation repeats another lesson's answer wording. Codex logged no tool call or output before the deadline, so its failure is unexplained rather than a known defect.

Run the measurement by hand only, from this repository and against a disposable Server: `target/debug/examples/working_set_measurement http://127.0.0.1:<port> <fresh absolute folder> <piper-voice.json> fixtures/coach/working-set.json [claude|codex]`.

## Remaining gaps

These are prototype measurements, not acceptance of the [new MVP](../specs/plan/implementation-plan.md). A killed prototype Coach may leave owned macOS children alive; do not assume those runs cleaned up. Existing CI and Windows checks do not establish native provider qualification. Completing the native Coach or expanding its qualification is not planned work.

## Manual acceptance checklist

Run attended checks only after authorization: Finder launch, keyboard and VoiceOver navigation, text scaling/themes, quit, exact generated reference listening, actual microphone/output and saved replay with Coach closed. Verify publication/reference/takes after browser reload and Server restart. Failed or unresolved attended checks remain explicit acceptance gaps; synthetic capture and WebKit do not establish physical-device behavior.

## Acceptance status

Silent software checks exercise the ordinary batch intake path with fake provider/speech boundaries, study, later independent checks, matched word-context rotation, scheduling and retained sessions/references/takes. They do not establish real model quality or speech quality. Inspect each real result and retain exact attempt identity, elapsed time, unknown usage where unavailable and failures; fresh semantic review and attended listening remain separate.

Mac Safari and iPhone recording and comparison playback were verified by the learner. Synthetic native-encoder tests inspect early, middle and late decoded signal plus original-byte/range delivery; they cannot establish physical microphone/output behavior. Use learner-triggered variable-length and successive takes, comparison and separately identified saved replay after reload/restart. These prototype observations do not establish the replacement's [recording acceptance](../specs/decision-recording-and-comparison.md#acceptance-boundary); check its saved replay and later real-clock return through the new ordinary workflow.
