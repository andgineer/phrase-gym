# Phrase Gym Coach desktop: development setup

Coach is not released. This page covers the development build, the platform facts measured so far, how to prepare the agent CLIs and what still has to be checked by a person. Product behavior lives in the [Coach decision](../specs/decision-external-preparation-agent.md); remaining work lives in the [Coach plan](../specs/plan/implementation-plan.md#4-integrate-the-minimal-coach-desktop-workstream).

## Build, run and smoke-test

| Task | Command |
|---|---|
| Run the Coach window from the checkout | `cargo run --package phrase-gym-coach --features desktop` |
| Build an unsigned development bundle | `cargo xtask coach-bundle [--out DIR]` |
| Build, launch like a user and verify the bundle | `cargo xtask coach-smoke` |

`coach-bundle` writes `Phrase Gym Coach.app` on macOS (the fake CLI used by the checks sits in `Contents/Resources`) and `Phrase Gym Coach\phrase-gym-coach.exe` with a `resources` folder on Windows. It is a development layout: unsigned, no icon, no installer.

The [functional MVP plan](../specs/plan/implementation-plan.md#functional-mvp-scope) selects Codex on one MacBook with full learning content and loop. The current bounded Claude path below is implemented development behavior, not that MVP. Wider platform and Claude qualification are future release work. Login commands below are explicit operator setup instructions; agents must ask the user to perform missing setup rather than automatically open login or credential-store dialogs.

## Local English lesson demo

Build once with `npm --prefix frontend run build` and `cargo xtask coach-bundle`. Then use these two commands in separate terminals on macOS:

```bash
cargo xtask run --data-dir /tmp/phrase-gym-preview
open "target/coach-bundle/Phrase Gym Coach.app"
```

Server and Coach connect directly on the same MacBook at `http://127.0.0.1:8000` by default; no connection setup, application authentication, token setup or Keychain access is used. An unavailable Server shows a recovery action. An optional local port override is described in the [local connection instructions](rust-preview.md). Remote operation and social sign-in remain later work.

For raw preparation, paste one exact fixed English phrase in **Material draft**, complete its title, situation/meaning, original performer and English audio locale, then choose or drop its matching original WAV clip (at most 1 MB). That deliberate audio action starts durable delivery and Claude preparation automatically. It needs the verified native Claude build and subscription login below. No separate Connect/Send action is part of ordinary intake. Ordinary input supports one outstanding local submission and one new English phrase; no source fetch, TTS, existing-target matching or additional CLI selection is included.

For deterministic supplied content without a model call, use **Choose prepared lesson…** and select the [sample prepared lesson](../fixtures/coach/supplied-lesson.json), or **Load supplied demo text**. Coach displays the phrase, situation, help and reference metadata. Loading validates locally and does not confirm publication. Choose or drop the matching original WAV to start automatic transfer/publication. **Preview selected original audio** plays the selected clip on macOS without publishing it. The fixture silence clip is only for software checks, never listening acceptance.

The pending envelope retains the exact original bytes and submission identity in `pending-send.json` under Coach's per-user state folder before transfer. Connection failures retry automatically with the same identity; the envelope stays bound to the original Server origin. Confirmed publication clears it. Accepted work belongs to Server; local staging is never described as accepted. A failed model attempt requires fixing the stated problem and explicitly choosing **Retry preparation after fixing the problem** (or retrying accepted browser input from its status panel). Authentication/quota failures never switch CLI or use an API key. **Retry connection and retained input** resumes after a recoverable connection/tool problem.

When Coach reports confirmed publication, open browser practice and choose **Start session** for ungraded study: read/hear, record, compare, replay and **Next** without rating memory. Browser reload and Server restart preserve published content, references and saved takes with Coach closed. Browser intake can submit the same bounded raw English form; the connected Coach polls accepted work automatically.

The generated path pins native Claude Code 2.1.302 and `haiku` selection, its immutable generation/profile/guide and the committed phrase output schema. Native installs retaining that measured version use it even when the public CLI symlink updates; another version fails closed pending isolation acceptance. The runner stages prompt/input/schema in a protected directory outside the repository, starts only the verified Read-only invocation with null stdin, caps each stdout/stderr stream at 64 KiB and the process at 90 seconds, and renews its Server lease every 30 seconds. It needs at least 110 seconds left in the Server attempt deadline before starting. The Server separately enforces three attempts and twenty starts per UTC day. A validated private result is reused after restart; an interrupted attempt without validated output is not started twice under the same fence. Server receipts reconcile ambiguous publication acknowledgement before any new preparation.

### No-spend Codex measurement preparation

After performing the attended Coach-home login below, prepare the first Codex measurement without starting a model:

```bash
cargo run --package phrase-gym-coach --example codex_preflight -- /tmp/phrase-gym-codex-fresh
```

The command derives Coach's own home from the OS per-user folders; there is no home override or execution option. It checks native Codex 0.159.2 and ChatGPT login using only bounded `--version` and `--ignore-user-config login status` processes with allowlisted environment and null stdin. Login status is inspected locally, never printed or retained. Missing setup, another version, API-key login, symlinked state/home, home-level instructions or custom skills fail actionably. Codex's bundled `skills/.system` is allowed; the existing invocation disables optional tools. The check never logs in, copies credentials or logs out an existing API login. It cannot establish isolation or language quality.

Use a fresh absolute directory with an existing parent outside repositories. On success it contains synthetic Unicode `input.txt`, `prompt.md` and `measurement.json` with the exact existing adapter arguments, environment names, Coach-home path and proposed limits. It contains no environment values or authentication output. The proposed first authorized measurement is one `gpt-5.6-luna` low-effort start, at most 90 seconds and 64 KiB per stream. The manifest is a review artifact, not an executable launcher; staging grants no spending permission.

Before real execution, agree an explicit maximum start/token-or-cost budget and stop conditions. Reuse the existing `cli_probe` cases after authorization rather than substituting a successful preflight for their results. Run `io`, `outside-read`, `outside-write`, `web`, `credentials` and `instructions` in Coach's logged-in home with fresh synthetic scratch directories. Inspect each result before the next start; stop on any unexpected access, missing denial, authentication/quota error, empty/truncated output or budget exhaustion. The credential case asks only whether access succeeded, never for credential contents; unexpected access fails acceptance. Do not print or preserve any accidentally returned secret. Record exact executable/version/configuration, case identity, limits, elapsed time, usage (unknown when absent), output acceptance/failure and OS permission evidence. A successful exit or model assertion alone is insufficient.

For package measurements, retain synthetic development fixtures separately from unseen held-out cases before spending. Cover fixed wording/alternatives, slots, situation and partner-turn prompts with answer-leak checks, words with sense cues/contexts/forms, preferred targets/confusables/dependencies/dialogue exposure, ambiguity/missing preferences, and exact supplied/missing/invalid audio. Cross English, German and Serbian with the specified card/help/translation preferences. Begin with one bounded English text-only case once the missing-speech path exists; unsupported contracts remain blocked rather than replaced with a simpler case. Keep schema conformance, fresh independent semantic review and attended listening as separate results for every output. Speech acceptance must cover each selected language/voice, Serbian conversion, names and foreign words; inspect primary-source availability, license and runtime requirements before selecting a configuration. No primary speech configuration or listening acceptance is established by this command.

A concrete initial fixture protocol (inputs are synthetic; none uses learner data):

| Case | Development input / expected review | Held-out input / expected review |
|---|---|---|
| English phrase | “Could you say that again?” / natural situation cue without answer leakage | “Would you mind speaking more slowly?” / same rules, unseen wording |
| Frame/alternatives | “Could I have [item], please?” / retain slot and valid alternatives | “I'd like [quantity] [item].” / distinguish frame from fixed wording |
| Word sense | “bank”, context “deposit money” / financial sense and 2–3 comprehensible contexts | “light”, context “not heavy” / correct adjective sense and applicable forms |
| Dialogue/exposure | A: “Tea or coffee?” B: “Tea, please.” / separate partner cue and response; declare exposure | A: “Window or aisle?” B: “Aisle, please.” / same evidence constraints |
| German | “Könnten Sie das bitte wiederholen?” / German speech, pinned help language | “Ich hätte gern ein Glas Wasser.” / German text and exact speech identity |
| Serbian | “Želim čaj.” / preserve display text, review speech conversion | “Đorđe želi kafu.” / names/digraphs and original script reviewed by listening |
| Reject/defer | “bank” without sense/context; absent required preferences / actionable retained input | URL-only input; invalid supplied WAV / reject rather than invent/fetch/replace |
| Identity/lifecycle | Two preferred alternatives plus a confusable and directed prerequisite / separate target histories | Re-preparation with changed meaning / no accidental identity reuse |

For each applicable case, run supplied-valid audio, genuinely missing audio, and invalid-supplied audio variants under separately bounded authorization. Rotate the required card/help/translation preference combinations across development and held-out sets. These rows define future measurements, not implemented contract coverage or permission to execute them.

### Bounded Piper speech proof

The Rust Coach library stages an explicitly configured Piper 1.8.0 process independently of the GUI and publication runner. It copies checksum-verified model/config bytes into a fresh private directory outside repositories, validates language/locale, a single speaker and sample rate, and stages exact speech text separately from displayed text. Metadata and the cache key include the exact texts, source, role, voice, locale, model/config hashes, configured runtime version, preparation version and synthesis settings. This is a cache identity, not a durable cache or a ready lesson. No download, provider switch, playback or publication occurs.

The separately installed Python interpreter must have an absolute path (`voice.executable`); preserve its virtual-environment path rather than resolving the executable symlink. Staging runs one bounded isolated `-I` metadata check without importing the speech engine: Piper must report 1.8.0 and ONNX Runtime 1.x. The observed package versions, Python version and interpreter binary SHA-256 enter the cache identity. Synthesis uses that same interpreter with `-I -m piper`. The check allows at most 15 seconds and 16 KiB per stream; it neither synthesizes nor certifies an installation against later external modification. Null stdin, an allowlisted environment, a 60-second deadline and a 2,000,000-byte limit per stream bound the process. Raw mono signed 16-bit little-endian PCM is captured through the existing process-tree supervisor, then wrapped as WAV using the verified config's sample rate. Empty, all-zero, odd-length, over-30-second or oversized PCM, failed/cancelled/truncated processes and unclosed streams are rejected. Model input is at most 128 MiB, config 64 KiB and each text 800 UTF-8 bytes. Staged files remain private disposable evidence. Raw output avoids an unbounded provider-created audio file. Framing and bounds cannot detect even-length logging that resembles PCM or certify matching speech/pronunciation; listening and Server media validation remain required.

[Piper's CLI](https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/CLI.md) supports explicit model/input files and raw output. Its [package metadata](https://github.com/OHF-Voice/piper1-gpl/blob/main/setup.py) requires Python 3.9+, uses ONNX Runtime and declares GPL-3.0-or-later. It embeds espeak-ng. Piper remains an external tool, with no speech runtime linked into Server or Coach. The project Python environment supplies it during migration; use a separately maintained installation after Python cutover. Engine licensing does not establish voice-dataset terms. The [Lessac medium card](https://huggingface.co/rhasspy/piper-voices/blob/main/en/en_US/lessac/medium/MODEL_CARD) points to the [Blizzard 2013 dataset research license](https://www.cstr.ed.ac.uk/projects/blizzard/2013/lessac_blizzard2013/license.html), which describes research restrictions on its materials. This inspection does not determine applicability to distributed weights or authorize accepting terms. The [Thorsten medium card](https://huggingface.co/rhasspy/piper-voices/blob/main/de/de_DE/thorsten/medium/MODEL_CARD) labels its dataset CC0 and reports fine-tuning from Lessac. Both voices remain provisional pending that review and listening.

For Serbian, [edge-tts](https://github.com/rany2/edge-tts) is an online service client; its [package metadata](https://github.com/rany2/edge-tts/blob/master/setup.cfg) declares LGPLv3 and Python 3.7+. [Microsoft's Azure voice table](https://learn.microsoft.com/en-us/azure/ai-services/speech-service/language-support?tabs=tts) lists Cyrillic `sr-RS-SophieNeural` and Latin `sr-Latn-RS-SophieNeural`. That table does not establish Edge availability. No Edge service/voice-list call or Serbian synthesis has run, and no Serbian primary is selected. Speech fixtures keep explicit conversion separate from display: automatic character replacement is insufficient for morpheme boundaries, names and foreign words.

Frozen [development](../fixtures/coach/speech/development.json) and [held-out](../fixtures/coach/speech/held-out.json) cases separate tuning from later evaluation. They contain synthetic text and listening criteria, never learner recordings or accepted speech. They cover only speech; full content/package measurements remain outstanding.

After explicit authorization for **one local synthesis**, use:

```sh
cargo run --package phrase-gym-coach --example piper_probe -- --authorized-local-synthesis /absolute/proof.json /absolute/existing/staging-parent
```

The argument acknowledges previously authorized execution; it grants no permission itself. `proof.json` contains `voice` (absolute Python `executable`, `model`, `config`, `runtime_version`, `model_sha256`, `config_sha256`, `voice`, `locale`) and `request` (`language`, exact `text`, exact `speech_text`, `source`, `role`). Start with development case `en-first`, `role: "reference"`, `source: "synthetic-speech-proof-v1"`. Stop after one attempt, including failure. Inspect metadata and every result before another attempt; playback/listening is separately attended. No real synthesis or listening has yet been accepted.

The first candidate needs authorization to obtain two pinned Lessac artifacts after voice-license review. At revision `c10ece1aade47bb51c153c893d14e5bf8e5b7117`, the [model](https://huggingface.co/rhasspy/piper-voices/resolve/c10ece1aade47bb51c153c893d14e5bf8e5b7117/en/en_US/lessac/medium/en_US-lessac-medium.onnx) is 63,201,294 bytes, SHA-256 `5efe09e69902187827af646e1a6e9d269dee769f9877d17b16b1b46eeaaf019f`; the [config](https://huggingface.co/rhasspy/piper-voices/resolve/c10ece1aade47bb51c153c893d14e5bf8e5b7117/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json) is 4,885 bytes, SHA-256 `efe19c417bed055f2d69908248c6ba650fa135bc868b0e6abb3da181dab690a0`. Sizes/model hash came from repository file metadata; the small config was fetched and hashed, while weights were not downloaded. Use the external Python interpreter with Piper 1.8.0 installed, voice `en_US-lessac-medium`, locale `en-US`; verify both downloaded hashes before execution. Proposed authorization: one offline CPU attempt, 60 seconds, 2,000,000 bytes per stream, at most 30 seconds of audio, no paid service/model CLI, followed by attended listening. This proposal authorizes no execution or automatic license acceptance.

### Claude no-spend measurement preparation

This command only stages the concrete [raw English fixture](../fixtures/coach/raw-english.json), copied synthetic silence clip, pinned prompt/profile/guide/schema and argument vector, and checks version/subscription login. It has no model execution option:

```bash
cargo run --package phrase-gym-coach --example prepare_dry_run -- /tmp/phrase-gym-fresh-dry-run
```

Choose a fresh absolute directory outside repositories. The proposed first measurement is one `haiku` start, 90 seconds maximum and 64 KiB per stream, followed by inspection of every output and a fresh semantic reviewer. Real spending needs separate authorization. The silence clip establishes transfer only; a matching spoken reference, attended listening/recording and actual-device acceptance remain required before claiming a real generated-lesson demonstration. This does not establish unattended language quality.

Headless deterministic transfer remains available:

```bash
cargo run --package phrase-gym-coach --example supplied_send -- \
  http://127.0.0.1:8000 \
  fixtures/coach/supplied-lesson.json /absolute/path/reference.wav /absolute/path/pending-send.json
```

Native fake-process tests and real-browser checks exercise production Coach/Server transfer and publication without provider calls. Packaged startup smoke exercises window/process/environment boundaries. These software checks do not establish attended native picker behavior, physical microphone/output, or language quality. Killed-Coach orphan recovery on macOS remains in the [remaining gaps](#remaining-gaps).

## Packaged startup smoke checks

`coach-smoke` places the bundle under a path with spaces and non-ASCII characters and starts it the way a person does, with a rebuilt environment. On macOS that is LaunchServices (`open`) with only the variables launchd gives a Finder launch, because `open` forwards its own environment. On Windows it is PowerShell 7 (`pwsh`) starting Coach with only the machine and user environment variables, the same block `Start-Process -UseNewEnvironment` builds. Both launches add three variables to the app's own environment: `PHRASE_GYM_LAUNCH_CANARY` plus fake `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` values, passed with `open --env` on macOS and set on the .NET process start on Windows; `Start-Process -Environment` silently dropped them when combined with `-UseNewEnvironment`. The app then writes a report, and the smoke test fails unless all of these hold:

- the real window opened on a hardware or software renderer;
- at most 2 frames were drawn during a 3-second idle period;
- the app received all three injected variables, and the bundled fake CLI it started saw none of them;
- the bundled fake CLI received Unicode arguments and its working directory, was cut off at the 64 KiB output limit and left no grandchild running;
- the per-user folders exist; startup performs no credential-store probe.

Keeping the launching terminal's variables away from the app is a property of the launcher itself, not of Coach. On Windows the PowerShell that starts Coach carries `PHRASE_GYM_TERMINAL_CANARY`, and the smoke test fails if the app sees it, which only confirms the environment was rebuilt; on macOS this is not tested. Real Finder and Start-menu launches are in the manual checklist.

The report also lists the Codex and Claude executables found without the shell's PATH, whether each is a native build, and the `--version` output of the native ones. Two failure paths are checked as well: a launch with no usable renderer (`WGPU_BACKEND=gl`) must exit with code 2 and an actionable message, and a launch while Coach's instance lock is held must exit with code 3. CI runs the smoke test on macOS Apple Silicon, macOS Intel and Windows x64.

## Platform targets and measured builds

| Component | macOS | Windows |
|---|---|---|
| Coach runtime: Rust 1.96, eframe/winit, wgpu with Metal or Direct3D 12 | 11+ on Apple Silicon (Rust target minimum); Metal needs 10.13+ on Intel | 10+ x64 (Rust target minimum, Direct3D 12) |
| [Claude Code](https://code.claude.com/docs/en/setup) | 13.0+ | 10 version 1809+ or Server 2019+ |
| [Codex CLI](https://github.com/openai/codex/blob/main/docs/install.md) | 12+ | officially Windows 11 via WSL2; a native PowerShell installer also exists |
| **Build targets** | **13.0+ on Apple Silicon and Intel** (`LSMinimumSystemVersion` 13.0) | **10 version 1809+ x64**, with the Codex gap below |

These build targets do not establish release support. The functional MVP requires Codex on the actual MacBook; a future release selects and qualifies its adapter/platform matrix. The table retains current build minimums; it does not make Claude a Codex MVP dependency. Native Windows Codex is unverified: its own documentation names WSL2, and Coach launches native executables only. Tested so far: macOS 27.0.1 on Apple Silicon by hand, plus the CI images listed below. The oldest claimed versions, macOS 13 and Windows 10 1809, are untested.

## Toolkit and measurements

egui, eframe and egui_kittest 0.37.0 with wgpu 30.0.1. Only the Metal and Direct3D 12 backends are compiled. AccessKit comes in through eframe's `accesskit` feature, and the status window prefers the low-power GPU. Measured on an Apple M1 Pro with macOS 27.0.1 and the release bundle:

| Measurement | Result |
|---|---|
| Idle CPU after startup | 0.0%: 0.06 s of CPU time in 60 s, 0 idle wake-ups |
| Redraws while idle | 2 frames in a 3-second window, 10 frames for the whole smoke run |
| Memory | about 100 MB resident (renderer included) |
| Size | 9.4 MiB release binary; 9.8 MiB development bundle including the 0.45 MiB fake CLI |

The packaged smoke test, including the injected-variable checks, passed in CI run 39029050142 (2026-10-17) on every runner, each with 2 idle frames:

| Runner image | Renderer found | Development bundle |
|---|---|---|
| `macos-26-arm64` (Apple Silicon VM) | Metal, Apple Paravirtual device | 9.8 MiB |
| `macos-15` on `macos-15-intel` (Intel VM) | Metal, Apple Paravirtual device | 10.7 MiB |
| `windows-2025-vs2026` (no GPU) | Direct3D 12, Microsoft Basic Render Driver (software) | 11.3 MiB |

Renderer fallback: none was added. Direct3D 12 starts even without a GPU through the Windows software rasterizer, and Metal starts in the macOS virtual machines. If the renderer cannot start, a native dialog explains the next step: update the graphics driver, or enable graphics acceleration in a Remote Desktop or virtual machine session.

## Per-user folders

| Purpose | macOS | Windows |
|---|---|---|
| Settings and instance lock | `~/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach` | `%APPDATA%\andgineer\Phrase Gym Coach\config` |
| Cache and attempt staging | `~/Library/Caches/io.github.andgineer.Phrase-Gym-Coach` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\cache` |
| Logs | `~/Library/Logs/Phrase Gym Coach` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\data\logs` |
| Coach's own Codex home | `~/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/codex-home` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\data\codex-home` |

Coach stores only nonsecret Server settings and never calls an OS credential store. A second Coach for the same user refuses to start and says one is already running.

## Agent CLI setup

Verified with Claude Code 2.1.302 and codex-cli 0.159.2 on macOS 27.0.1 (Apple Silicon) on 2026-10-17. The exact invocation and the evidence behind each flag are in [Verified CLI invocation](#verified-cli-invocation) below.

**Supported installs.** Only native builds are verified: Claude Code from its native installer (`~/.local/bin/claude`) and Codex from the Homebrew cask (`brew install --cask codex`). Coach starts a CLI with a PATH of only the executable's folder and the system folders, so an install that is a Node script fails inside an attempt with `env: node: No such file or directory` (exit 127). That covers `npm install -g @openai/codex` (which runs `bin/codex.js`), `npm install -g @anthropic-ai/claude-code` (which runs `cli.js`) and the older `~/.claude/local/claude` wrapper. The status window reports such an install as unsupported rather than found, and Coach does not look in `~/.claude/local`. Script installs remain unsupported; adding an explicit runtime is future adapter work, not a prerequisite for the native Codex MVP.

**Claude CLI** uses the existing subscription login. Check it without using the model: `claude auth status` must report `"loggedIn": true` and `"authMethod": "claude.ai"`. Coach strips `ANTHROPIC_API_KEY` and similar variables, so an exported key never takes over.

**Codex CLI** gets a separate Codex home that belongs to Coach, so the instructions (`AGENTS.md`) and skills in your normal `~/.codex` never reach a job. Coach creates that folder each time it starts: owner-only on macOS, and on Windows it inherits your profile's access list. Codex refuses a home that does not exist yet (`CODEX_HOME points to "…", but that path does not exist`, exit 1), so the commands below create it first and work before Coach has ever run. Sign in once with your ChatGPT plan:

```bash
COACH_CODEX_HOME="$HOME/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/codex-home"
mkdir -p "$COACH_CODEX_HOME" && chmod 700 "$COACH_CODEX_HOME"
CODEX_HOME="$COACH_CODEX_HOME" codex login
CODEX_HOME="$COACH_CODEX_HOME" codex login status
```

On Windows, where native Codex is still unverified (see [Remaining gaps](#remaining-gaps)), the PowerShell equivalent is:

```powershell
$coachCodexHome = "$env:LOCALAPPDATA\andgineer\Phrase Gym Coach\data\codex-home"
New-Item -ItemType Directory -Force -Path $coachCodexHome | Out-Null
$env:CODEX_HOME = $coachCodexHome; codex login; codex login status; Remove-Item Env:CODEX_HOME
```

The status must read `Logged in using ChatGPT`. Coach forces ChatGPT login, so an API-key login in that home is refused and logged out instead of being billed.

What a job can and cannot do, checked with synthetic canary files and the cheapest models (Claude `haiku`, Codex `gpt-5.6-luna` at low effort):

| Check | Claude CLI | Codex CLI |
|---|---|---|
| Read the staged input in the attempt folder | allowed, exact text returned | allowed, exact text returned |
| Read a canary file outside the attempt folder | denied by the CLI ("confines the file tools to the working directory") | denied by the sandbox ("Operation not permitted") |
| Write outside the attempt folder | impossible: Read is the only tool | denied by the sandbox; nothing written |
| Web access | no web tool | web search disabled; `curl` cannot resolve hosts |
| Read the CLI's own credential/config files | `~/.claude.json` denied (the login itself is in the Keychain) | `~/.codex/auth.json` denied |
| User instructions, MCP servers, plugins, hooks, connectors | none loaded: only the Read tool, no MCP servers, skills or slash commands | none from `config.toml`; tools limited to command execution and patching. Bundled system skills are still listed |
| Launch from a LaunchServices-started app without the shell's PATH | works | works |

Each real call took 3 to 12 seconds. A trivial Codex attempt reports about 8,000 tokens; Claude reports no usage in text mode.

## Verified CLI invocation

The argument and environment builders in `crates/phrase-gym-coach/src/adapter/` (`claude.rs`, `codex.rs` and `environment.rs`) are the only code that knows these flags. Their unit tests pin the complete argument vectors below, and `examples/cli_probe/` repeats the real probes. Re-run the probes whenever a CLI version or any flag changes. Everything here was verified on 2026-10-17 with Claude Code 2.1.302 and codex-cli 0.159.2 on macOS 27.0.1 (Apple Silicon), using synthetic canaries, Claude `haiku` and Codex `gpt-5.6-luna` at low effort. The prompt-file, stdin, `--`, `--bare`, Claude tool and environment findings come from `news-recap` and are settled; do not rediscover them.

### Every attempt

- **Executable.** Resolve the canonical path, following symlinks, from the documented install locations (`~/.local/bin`, `/opt/homebrew/bin`, `/usr/local/bin`, `%USERPROFILE%\.local\bin`) and then the GUI's PATH; never rely on shell initialisation. A native build found later in that order wins over a script found earlier. Claude's native installer keeps only the two newest versions, so recheck the path before each attempt.
- **Attempt folder.** A fresh folder per attempt under the Coach cache holds `prompt.md`, the staged inputs and bounded `stdout.log`/`stderr.log`. It is also the working directory, so no repository instructions or git context load.
- **Prompt delivery.** The full prompt stays in the file. The only prompt argument is `Read your task from <absolute prompt path> and execute it.`, and stdin is null. Never use stdin: `claude -p` returns empty output once piped stdin exceeds a few KB ([anthropics/claude-code#7263](https://github.com/anthropics/claude-code/issues/7263)), and `codex exec` with a non-TTY stdin hangs waiting for EOF ([openai/codex#20919](https://github.com/openai/codex/issues/20919)). With null stdin, Codex prints `Reading additional input from stdin...` and continues.
- **Environment.** Built from an allowlist; nothing else is inherited. On macOS: `HOME USER LOGNAME SHELL TMPDIR LANG LC_ALL LC_CTYPE`. On Windows: system and profile folders, `TEMP`/`TMP`, user and computer names and processor variables. `PATH` is the executable's folder plus the system folders, which is why Node-script installs cannot start. Everything else is dropped, including `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`, `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `CODEX_API_KEY`, proxies and parent agent session variables (`CLAUDECODE`, `CLAUDE_CODE_*`). With `ANTHROPIC_API_KEY` present, `claude auth status` reports API-key auth; with the allowlist it reports the claude.ai subscription.
- **Supervision.** Each attempt has a deadline. A deadline, cancellation or output overflow sends TERM to the process group and KILL after a grace period (on Windows, the Job Object is terminated). Descendants are killed after a normal exit too.
- **Answer.** Plain stdout is the default answer channel and is proven for both CLIs. Exit 0 with empty stdout is a failure ("no output produced").

### Readiness, failures and usage

- **Readiness without model usage.** `claude auth status` must show `"loggedIn": true` and `"authMethod": "claude.ai"`; `codex login status` must print `Logged in using ChatGPT`.
- **Recorded failure outputs.** Claude without a login prints `Not logged in · Please run /login` to stdout and exits 1. Codex without a login retries five times, then prints `unexpected status 401 Unauthorized`. Codex refusing an API-key login prints `ChatGPT login is required, but an API key is currently being used. Logging out.` Codex pointed at a missing home prints `CODEX_HOME points to "…", but that path does not exist` and exits 1.
- **Quota and rate limits.** Reuse the `news-recap` patterns over both stderr and stdout: `rate.?limit`, `too many requests`, `\b(status|code|http)\b\W{0,10}429`, `credit balance`, `Individual quota reached|upgrade your subscription` and `overloaded`. Never match a bare `429`: it also matches PIDs and trace ids.
- **Classifying a failure.** Use structured status where the CLI provides it (the exit status, Claude's JSON `is_error`, Codex's `--json` events) and bounded, adapter-specific matching of the recorded outputs above otherwise. Startup noise is not a final failure: with null stdin Codex prints `Reading additional input from stdin...`, it warns on stderr about unknown `-c` keys, and without a login it retries five times before the final 401 recorded above. `news-recap` likewise ignores an early "not logged in" line once a later line reports successful authentication. Report a quota reset time only when the CLI prints one; never invent or estimate it.
- **Structured output.** Claude `--output-format json` adds `result`, `is_error`, `permission_denials` and usage, and is the candidate for normalising results in the remaining integration work; Claude's text mode reports no usage. Codex prints `tokens used\n<total>` to stderr; `--json` adds per-turn input, cached, output and reasoning tokens; `--output-last-message <file>` only duplicates the final stdout message.

**Sources.** The `news-recap` evidence is read-only under `../news-recap/`: `src/news_recap/recap/agents/{ai_agent,subprocess,routing}.py`, `recap/contracts.py`, the command defaults in `src/news_recap/config.py`, `tests/recap/agents/`, `spec/agents.md` and `spec/plan-agent-sandboxing.md`. Its code and tests are the evidence, not its specs: they describe running in the project root, while the code runs in a temporary directory and adds Codex's non-repository flag. The code also copies the parent environment, its command defaults grant broad tools and network access, and termination reaches only the immediate child; Coach adopts none of these. A prompt asking an agent to stay inside its folder is not enforcement; only the CLI permission checks recorded here count.

### Claude

```text
<claude> -p --model <model> [--effort <level>] --safe-mode --restricted --strict-mcp-config
  --tools Read --allowed-tools Read --permission-prompts none --no-session-persistence
  --disable-slash-commands -- "Read your task from <prompt> and execute it."
env += CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 ENABLE_CLAUDEAI_MCP_SERVERS=false
```

- `--restricted` ignores user, project and local settings (permission allow lists, enabled plugins) and the user's `CLAUDE.md`, removes code-running tools and confines file tools to the working directory: reading a canary or `~/.claude.json` outside it fails with "--restricted confines the file tools to the working directory". Neither `--tools Read` nor `--safe-mode` replaces it: `--tools Read` alone still loads the user's `CLAUDE.md`, and `--safe-mode` alone still offers Bash, WebFetch and Write. With `--strict-mcp-config` as well, the session init lists only the Read tool, no MCP servers, skills or slash commands, two built-in plugins and `apiKeySource: none`.
- `--` is required because `--allowed-tools` is variadic and would otherwise swallow the bootstrap. `--allowed-tools Read` lets Read run without a prompt; `--permission-prompts none` denies anything that would prompt.
- The environment additions stop background self-updates and telemetry during a pinned attempt, and stop the fetching of claude.ai connectors (remote MCP servers).
- `haiku` resolves to Claude Haiku 5.5, whose thinking `MAX_THINKING_TOKENS=0` cannot disable; `CLAUDE_CODE_MAX_OUTPUT_TOKENS=64000` remains the knob for large outputs. Never set `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING` alone: it pushes reasoning into stdout on models that honour it.
- Never pass `--bare` (it limits authentication to `ANTHROPIC_API_KEY`, so the subscription login is ignored), `--permission-mode dontAsk|bypassPermissions`, `--dangerously-skip-permissions`, the WebFetch or Bash tools, or `--add-dir`.

### Codex

```text
<codex canonical path> exec --skip-git-repo-check --ignore-user-config --ignore-rules --ephemeral
  -c default_permissions="coach_job"
  -c permissions.coach_job={filesystem={":root"="deny", ":minimal"="read", ":slash_tmp"="deny",
       ":tmpdir"="deny", "<codex install dir>"="read", ":workspace_roots"={"."="write"}},
       network={enabled=false}}
  -c forced_login_method="chatgpt" -c approval_policy="never" -c web_search="disabled"
  -c history.persistence="none" -c check_for_update_on_startup=false
  --disable apps --disable browser_use --disable browser_use_external --disable computer_use
  --disable goals --disable hooks --disable image_generation --disable in_app_browser
  --disable multi_agent --disable plugins --disable remote_plugin
  --disable skill_mcp_dependency_install --disable tool_suggest --disable view_image
  --model <model> -c model_reasoning_effort=<level> "Read your task from <prompt> and execute it."
env += CODEX_HOME=<Coach-owned Codex home>
```

- A permission profile replaces `--sandbox workspace-write`: that legacy sandbox lets commands read the whole disk and write `/tmp`, and passing any `--sandbox` makes Codex ignore permission profiles. `codex sandbox -P` showed that `:minimal` opens writes to `/tmp` and `$TMPDIR` unless they are denied explicitly, and that the sandbox helper re-executes Codex by its invoked path. The install folder therefore needs read access, and Codex must be launched by its canonical path; a rule on the Homebrew symlink is not enough. With the profile, reading or writing canaries outside the attempt and reading `~/.codex/auth.json` fail with "Operation not permitted", and `curl` cannot resolve hosts.
- `--ignore-user-config` drops `config.toml` (MCP servers, plugins, `notify`, model, developer instructions) but still loads `$CODEX_HOME/AGENTS.md` and `$CODEX_HOME/skills`, shown by capturing the request of a synthetic home against a local endpoint. That is why Coach owns a separate Codex home with its own ChatGPT login. Bundled system skills (imagegen, openai-docs, skill-creator, skill-installer) still appear; the remaining tools are command execution, stdin writes, patching, waiting and user-input requests. `view_image` (it reads files from the unsandboxed Codex process) and goals (they persist state in the Codex home) are disabled.
- `forced_login_method="chatgpt"` refuses and logs out an API-key login; without it, a stored API key is used.
- `--skip-git-repo-check` is required because the attempt folder is not a git repository.
- An unknown `--disable` name fails the launch, which surfaces version drift; unknown `-c` keys are only ignored with a warning on stderr.

## Remaining gaps

- **Codex home login.** The probes used the operator's own `~/.codex` with user configuration ignored. The no-spend Codex preflight above checks setup only. A run inside Coach's own Codex home still needs your one-time login, explicit measurement budget authorization and the confirming probes in the checklist below.
- **Windows CLIs.** No Windows probes have run with logged-in CLIs. Still unverified there: native Codex support and whether its permission profile is enforced, Claude's `--restricted` confinement, and where the tools are installed. CI covers only the build, the UI tests, process cleanup through the Job Object (including grandchildren and dropping the supervisor), credential-store-free startup and the packaged smoke test.
- **Crash cleanup on macOS.** If Coach itself is killed, its CLI children survive, because macOS has nothing like the Windows kill-on-close Job Object. Recovery after a crash belongs to the runner work.
- **Claude updates.** Claude's native installer keeps only the two newest versions, so the version Coach found can disappear after updates. Coach must recheck the tool before every attempt.

## Manual acceptance checklist

These are attended operator checks, not instructions to launch prompts automatically. Run applicable checks with the bundle from `cargo xtask coach-bundle` and record OS/version/results. The functional MVP requires the actual MacBook/Codex workflow; Windows, Intel, oldest-version and Claude checks are deferred release qualification.

1. **Finder launch (macOS).** Double-click `target/coach-bundle/Phrase Gym Coach.app`. The window opens and every status row stops saying "Checking…" within a few seconds. Codex and Claude show the paths found. A second double-click shows the "already running" dialog.
2. **VoiceOver.** Turn it on with Cmd+F5. The heading, every status row, the theme radio buttons, the three text-size buttons and the "Material draft" field are announced by name. Typing German and Serbian Latin and Cyrillic text into the draft is read back correctly.
3. **Keyboard.** Tab reaches every control with a visible focus ring, Space selects a theme, and Cmd+plus and Cmd+minus resize the text.
4. **Display.** Light, Dark and System themes stay readable. The window can be resized down to its minimum without clipped controls, and it stays crisp on Retina and external displays.
5. **Local connection.** With Server and Coach on the same MacBook using their default port, verify automatic connection and recovery after Server outage. Coach must never display an application login or OS credential-store prompt.
6. **Windows.** Start the portable build from a Start-menu or desktop shortcut and repeat steps 1 to 4, using Narrator (Ctrl+Win+Enter) instead of VoiceOver and checking 150% and 200% display scaling. Then start it inside a Remote Desktop session and on a machine without Direct3D 12: it either opens or shows the renderer dialog.
7. **Intel Mac.** Repeat steps 1 and 2 on an Intel Mac.
8. **Oldest versions.** Repeat steps 1 to 3 on macOS 13 and on Windows 10 version 1809.
9. **Codex in Coach's home.** After the one-time login above, run `cargo run --package phrase-gym-coach --example cli_probe -- codex gpt-5.6-luna io "<scratch folder>" "<Coach codex-home>"`. It must print the staged line, `canary leaked: false` and `outside file written: false`. Repeat with the `outside-read` probe, which must print `canary leaked: false` and a reply starting with `DENIED`, and with the `credentials` probe, which must print `credential target: <Coach codex-home>/auth.json` and a reply starting with `DENIED`. The credentials probe exits with an error instead of running when that file does not exist, so a missing login cannot pass as a denial.
10. **Windows CLIs.** On a Windows machine with both CLIs logged in, run the probe example for `io`, `outside-read`, `outside-write`, `web` and `credentials` for each CLI, passing Coach's Codex home from the PowerShell setup above. Expect the same output as in step 9, with `outside file written: false` for `outside-write` and a reply starting with `NO-WEB` for `web`. The Claude `credentials` probe must name `%USERPROFILE%\.claude\.credentials.json` as its target (shown with `~` for the profile folder) and get `DENIED`; Codex must name `<Coach codex-home>\auth.json`.
