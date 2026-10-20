# Phrase Gym Coach desktop: development setup

Coach is not released. This page covers the development build, the platform facts measured so far, how to prepare the agent CLIs and what still has to be checked by a person. Product behavior lives in the [Coach decision](../specs/decision-external-preparation-agent.md); remaining work lives in the [Coach plan](../specs/plan/implementation-plan.md#coach-desktop-workstream).

## Build, run and smoke-test

| Task | Command |
|---|---|
| Run the Coach window from the checkout | `cargo run --package phrase-gym-coach --features desktop` |
| Build an unsigned development bundle | `cargo xtask coach-bundle [--out DIR]` |
| Build, launch like a user and verify the bundle | `cargo xtask coach-smoke` |

`coach-bundle` writes `Phrase Gym Coach.app` on macOS (the fake CLI used by the checks sits in `Contents/Resources`) and `Phrase Gym Coach\phrase-gym-coach.exe` with a `resources` folder on Windows. It is a development layout: unsigned, no icon, no installer.

## Supplied-lesson local preview

Coach can send one supplied prepared English phrase and one original reference clip to the existing Server inbox. This path transfers reviewed content; it does not call an agent or speech provider. The sample [prepared lesson](../fixtures/coach/supplied-lesson.json) contains the phrase “Let's get started.” and its situation/help. Supply a matching WAV clip of that phrase, at most 1 MB. The preservation silence fixture is suitable only for automated transfer checks, not listening acceptance.

1. Build the client with `npm --prefix frontend run build`, then start Server with a disposable data directory: `cargo xtask run --port 8000 --data-dir /tmp/phrase-gym-preview --credentials-file /absolute/path/server-credentials.json`. Provision separate `material_submit` and `worker` actors/tokens in that private credential file, plus a learner credential for browser login. Follow the [credential format](rust-preview.md); keep these private files outside Git.
2. Build `cargo xtask coach-bundle`, then open `target/coach-bundle/Phrase Gym Coach.app` on macOS. Enter the loopback Server URL and its material-submit and worker tokens, then click **Connect**. Tokens are stored in Keychain under separate accounts for that Server URL; leave them blank to reuse saved tokens.
3. Paste the sample JSON into **Material draft**, or enter/drop its absolute path into **Prepared lesson file** and click **Load prepared lesson**. Enter/drop the matching clip's absolute path into **Reference audio file**. Loading or dropping sends nothing.
4. Click **Send supplied lesson**. Coach retains the exact bytes and submission identity before upload, then displays durable acceptance, preparation and publication phases. **Retry pending Send** retains that identity after a lost response or restart. Connect to the same original Server URL before retrying. One outstanding send is supported; other accepted inbox work may keep this lesson waiting.
5. Click **Open browser practice**, sign in with the learner token if required, and choose **Start session** for ungraded study to hear/read, record, compare and replay, then use **Next** without rating memory. The browser card and stored media remain Server-owned when Coach exits.

The temporary sender envelope is `pending-send.json` in Coach's per-user data directory. It contains supplied input/audio, never tokens, and is cleared after a confirmed publication. Preserve it if a write fails. This preview starts no background polling or automatic preparation; retry is explicit. Its connection accepts loopback HTTP only and follows no redirects. General source import, model generation and release recovery remain deferred.

The retained headless probe uses the same transfer runner, with a private JSON object containing `material_submit` and `worker` token values:

```bash
cargo run --package phrase-gym-coach --example supplied_send -- \
  http://127.0.0.1:8000 /absolute/path/coach-credentials.json \
  fixtures/coach/supplied-lesson.json /absolute/path/reference.wav /absolute/path/pending-send.json
```

The automated process-boundary test builds and starts the production Server on disposable data, publishes through Coach and reconciles an identical sender envelope after Server restart. It uses synthetic silence and does not establish speech or physical microphone quality. The packaged interactive Send and attended feedback checks remain separate acceptance work.

## Packaged startup smoke checks

`coach-smoke` places the bundle under a path with spaces and non-ASCII characters and starts it the way a person does, with a rebuilt environment. On macOS that is LaunchServices (`open`) with only the variables launchd gives a Finder launch, because `open` forwards its own environment. On Windows it is PowerShell 7 (`pwsh`) starting Coach with only the machine and user environment variables, the same block `Start-Process -UseNewEnvironment` builds. Both launches add three variables to the app's own environment: `PHRASE_GYM_LAUNCH_CANARY` plus fake `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` values, passed with `open --env` on macOS and set on the .NET process start on Windows; `Start-Process -Environment` silently dropped them when combined with `-UseNewEnvironment`. The app then writes a report, and the smoke test fails unless all of these hold:

- the real window opened on a hardware or software renderer;
- at most 2 frames were drawn during a 3-second idle period;
- the app received all three injected variables, and the bundled fake CLI it started saw none of them;
- the bundled fake CLI received Unicode arguments and its working directory, was cut off at the 64 KiB output limit and left no grandchild running;
- the per-user folders exist and the credential store answered a read-only probe.

Keeping the launching terminal's variables away from the app is a property of the launcher itself, not of Coach. On Windows the PowerShell that starts Coach carries `PHRASE_GYM_TERMINAL_CANARY`, and the smoke test fails if the app sees it, which only confirms the environment was rebuilt; on macOS this is not tested. Real Finder and Start-menu launches are in the manual checklist.

The report also lists the Codex and Claude executables found without the shell's PATH, whether each is a native build, and the `--version` output of the native ones. Two failure paths are checked as well: a launch with no usable renderer (`WGPU_BACKEND=gl`) must exit with code 2 and an actionable message, and a launch while Coach's instance lock is held must exit with code 3. CI runs the smoke test on macOS Apple Silicon, macOS Intel and Windows x64.

## Supported platforms

| Component | macOS | Windows |
|---|---|---|
| Coach runtime: Rust 1.96, eframe/winit, wgpu with Metal or Direct3D 12 | 11+ on Apple Silicon (Rust target minimum); Metal needs 10.13+ on Intel | 10+ x64 (Rust target minimum, Direct3D 12) |
| [Claude Code](https://code.claude.com/docs/en/setup) | 13.0+ | 10 version 1809+ or Server 2019+ |
| [Codex CLI](https://github.com/openai/codex/blob/main/docs/install.md) | 12+ | officially Windows 11 via WSL2; a native PowerShell installer also exists |
| **Coach claims** | **13.0+ on Apple Silicon and Intel** (`LSMinimumSystemVersion` 13.0) | **10 version 1809+ x64**, with the Codex gap below |

A supported release needs both CLI adapters, so the macOS floor follows Claude Code. Native Windows Codex is unverified: its own documentation names WSL2, and Coach launches native executables only. Tested so far: macOS 27.0.1 on Apple Silicon by hand, plus the CI images listed below. The oldest claimed versions, macOS 13 and Windows 10 1809, are untested.

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

## Per-user folders and credentials

| Purpose | macOS | Windows |
|---|---|---|
| Settings and instance lock | `~/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach` | `%APPDATA%\andgineer\Phrase Gym Coach\config` |
| Cache and attempt staging | `~/Library/Caches/io.github.andgineer.Phrase-Gym-Coach` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\cache` |
| Logs | `~/Library/Logs/Phrase Gym Coach` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\data\logs` |
| Coach's own Codex home | `~/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/codex-home` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\data\codex-home` |

Secrets go only to the login Keychain or Windows Credential Manager, under the service `io.github.andgineer.phrase-gym-coach`. At startup Coach only reads that store. If access is denied, the status window says so, and Coach never saves the secret anywhere else. A second Coach for the same user refuses to start and says one is already running.

## Agent CLI setup

Verified with Claude Code 2.1.302 and codex-cli 0.159.2 on macOS 27.0.1 (Apple Silicon) on 2026-10-17. The exact invocation and the evidence behind each flag are in [Verified CLI invocation](#verified-cli-invocation) below.

**Supported installs.** Only native builds are verified: Claude Code from its native installer (`~/.local/bin/claude`) and Codex from the Homebrew cask (`brew install --cask codex`). Coach starts a CLI with a PATH of only the executable's folder and the system folders, so an install that is a Node script fails inside an attempt with `env: node: No such file or directory` (exit 127). That covers `npm install -g @openai/codex` (which runs `bin/codex.js`), `npm install -g @anthropic-ai/claude-code` (which runs `cli.js`) and the older `~/.claude/local/claude` wrapper. The status window reports such an install as unsupported rather than found, and Coach does not look in `~/.claude/local`. Script installs stay unsupported until the C3 settings in the [Coach plan](../specs/plan/implementation-plan.md#c3-implement-the-minimal-v1-surface) add an explicit runtime.

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
- **Structured output.** Claude `--output-format json` adds `result`, `is_error`, `permission_denials` and usage, and is the candidate for normalising results in C4; Claude's text mode reports no usage. Codex prints `tokens used\n<total>` to stderr; `--json` adds per-turn input, cached, output and reasoning tokens; `--output-last-message <file>` only duplicates the final stdout message.

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

- **Codex home login.** The probes used the operator's own `~/.codex` with user configuration ignored. A run inside Coach's own Codex home needs your one-time login, followed by the confirming probe in the checklist below.
- **Windows CLIs.** No Windows probes have run with logged-in CLIs. Still unverified there: native Codex support and whether its permission profile is enforced, Claude's `--restricted` confinement, and where the tools are installed. CI covers only the build, the UI tests, process cleanup through the Job Object (including grandchildren and dropping the supervisor), Windows Credential Manager and the packaged smoke test.
- **Crash cleanup on macOS.** If Coach itself is killed, its CLI children survive, because macOS has nothing like the Windows kill-on-close Job Object. Recovery after a crash belongs to the runner work.
- **Claude updates.** Claude's native installer keeps only the two newest versions, so the version Coach found can disappear after updates. Coach must recheck the tool before every attempt.

## Manual acceptance checklist

Run these with the bundle from `cargo xtask coach-bundle`. Record the OS version and the result of each step.

1. **Finder launch (macOS).** Double-click `target/coach-bundle/Phrase Gym Coach.app`. The window opens and every status row stops saying "Checking…" within a few seconds. Codex and Claude show the paths found. A second double-click shows the "already running" dialog.
2. **VoiceOver.** Turn it on with Cmd+F5. The heading, every status row, the theme radio buttons, the three text-size buttons and the "Material draft" field are announced by name. Typing German and Serbian Latin and Cyrillic text into the draft is read back correctly.
3. **Keyboard.** Tab reaches every control with a visible focus ring, Space selects a theme, and Cmd+plus and Cmd+minus resize the text.
4. **Display.** Light, Dark and System themes stay readable. The window can be resized down to its minimum without clipped controls, and it stays crisp on Retina and external displays.
5. **Credential-store denial.** Lock the login keychain (`security lock-keychain`), start Coach and cancel the unlock prompt. The status shows the credential store as unavailable or denied. Nothing is written to the settings folder.
6. **Windows.** Start the portable build from a Start-menu or desktop shortcut and repeat steps 1 to 4, using Narrator (Ctrl+Win+Enter) instead of VoiceOver and checking 150% and 200% display scaling. Then start it inside a Remote Desktop session and on a machine without Direct3D 12: it either opens or shows the renderer dialog.
7. **Intel Mac.** Repeat steps 1 and 2 on an Intel Mac.
8. **Oldest versions.** Repeat steps 1 to 3 on macOS 13 and on Windows 10 version 1809.
9. **Codex in Coach's home.** After the one-time login above, run `cargo run --package phrase-gym-coach --example cli_probe -- codex gpt-5.6-luna io "<scratch folder>" "<Coach codex-home>"`. It must print the staged line, `canary leaked: false` and `outside file written: false`. Repeat with the `outside-read` probe, which must print `canary leaked: false` and a reply starting with `DENIED`, and with the `credentials` probe, which must print `credential target: <Coach codex-home>/auth.json` and a reply starting with `DENIED`. The credentials probe exits with an error instead of running when that file does not exist, so a missing login cannot pass as a denial.
10. **Windows CLIs.** On a Windows machine with both CLIs logged in, run the probe example for `io`, `outside-read`, `outside-write`, `web` and `credentials` for each CLI, passing Coach's Codex home from the PowerShell setup above. Expect the same output as in step 9, with `outside file written: false` for `outside-write` and a reply starting with `NO-WEB` for `web`. The Claude `credentials` probe must name `%USERPROFILE%\.claude\.credentials.json` as its target (shown with `~` for the profile folder) and get `DENIED`; Codex must name `<Coach codex-home>\auth.json`.
