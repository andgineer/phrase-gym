# Remaining Coach desktop implementation

Build Phrase Gym Coach as a Rust GUI application for Windows and macOS. This is the Coach portion of [migration batch 6](implementation-plan.md#6-integrate-the-retained-vue-client-and-rust-coach), not a separate backend rewrite. The [Coach decision](../decision-external-preparation-agent.md) owns product behavior, permissions and lifecycle; [architecture](../architecture.md) owns data and dependency boundaries. Remove completed tasks here and retain lasting decisions in those documents. This plan does not implement the app or authorize provider spending, source-project edits, publication or deployment.

## Dependencies and first delivery

The automatable part of the desktop boundary is in place: the Coach crate, packaged smoke test, process supervision, platform adapters and the [proven CLI invocation contract](#proven-cli-invocation-contract); [setup documentation](../../docs/coach-desktop.md) records the measurements, minimum OS versions and manual checklist. C1 keeps only its open acceptance gaps. Independent Server and fake-runner work can continue while an adapter or platform gap is open; adapter-dependent runner and settings work in C2–C3 waits for the Codex home login probe. C2–C3 also require the contracts, profiles and durable job APIs from migration batches 3–4; contract fixtures can support development before the real routes exist, but cannot satisfy integration acceptance. C4 requires the revised package format. C5 integrates with the retained Vue client and Rust Server in batch 6. C6 packaging acceptance runs on the transition tree before the Python cutover in batch 7; batch 7 owns the final checks after Python removal. Real language/speech assessment remains in batch 8, and private operation in batch 9.

Target Windows x86_64 with MSVC and macOS Apple Silicon plus Intel, at the minimum OS versions documented in the setup notes. Do not imply Windows ARM, Linux desktop or every adapter supports these targets merely because Rust can compile for them. A supported release needs verified Codex and Claude CLI adapters and a speech configuration per claimed platform; otherwise that platform remains an explicit acceptance gap.

V1 has only an inbox drop area, settings selecting Codex or Claude CLI, and basic status/database indicators. Detailed GUI design is deferred: no queue browser, job-detail screen, profile editor, charts or diagnostics dashboard. The first usable slice is packaged launch → configure Server and selected CLI → drop/paste synthetic material → Send → durable inbox receipt → fake preparation/speech → atomic publication → updated status/counts. Keep tray operation, login startup, automatic updates, source import and a native practice client outside this scope.

## C1. Prove the desktop boundary on both operating systems

Automated coverage exists for every C1 item that a test or CI runner can reach; these gaps remain:

- Run the [manual checklist](../../docs/coach-desktop.md#manual-acceptance-checklist): Finder and Start-menu launch by a person, VoiceOver and Windows Narrator with the actual window, 150%/200% scaling, Remote Desktop/no-Direct3D-12 startup and an Intel Mac. Stop UI expansion and revise the toolkit decision if accessibility or rendering proves unusable; add a renderer fallback only if this evidence calls for it.
- Test the oldest claimed OS versions (macOS 13, Windows 10 1809); CI images are newer.
- Log in once to Coach's own Codex home and repeat the `io`, `outside-read` and `credentials` probes there; the recorded Codex probes used the operator's home with user configuration ignored.
- Run both real CLI adapters on Windows with logged-in CLIs: native Codex (its documentation names WSL2), its permission-profile enforcement, Claude `--restricted` confinement and Windows install locations are unverified. GitHub runners cannot hold CLI logins.

**Exit:** a minimal real window and fake worker run on all claimed architectures; both real CLI adapters pass the authentication/isolation feasibility gate on each claimed platform, and an independent Server build excludes Coach dependencies. Unresolved native or adapter constraints remain explicit acceptance gaps and block dependent work.

## C2. Build runner orchestration and recovery independently of the GUI

| Responsibility | Boundary |
|---|---|
| UI state and views | Typed commands and immutable snapshots; no HTTP, subprocess launch or blocking filesystem work inside rendering. Keep drop input, minimal settings and status presentation separate from orchestration. |
| Runner | Explicit processing state machine, one entry/group at a time, budgets, cancellation and orchestration. No egui types. |
| Server client | Scoped authentication, protocol compatibility, bounded reads/uploads, idempotency, lease renewal and receipt reconciliation using shared contracts. |
| Agent/speech adapters | Tool capability checks, request/output mapping and provider-specific failure classification. No queue ownership. |
| Process supervisor | Spawn, stream draining/limits, timeouts, cancellation, descendant termination and reaping through OS-specific implementations. |
| Local platform adapters | Credential store, paths, instance guard and redacted diagnostics; staging is disposable. |

- Run Tokio work outside the native UI thread, using bounded command/event channels and a latest-state snapshot for progress. Do not lose cancellation or final outcomes when progress traffic fills a channel. Fence UI responses by connection/configuration identity so an old response cannot overwrite a newly selected Server. Construct adapters at startup; apply the repository's module-size and inward-dependency rules.
- Model disconnected, paused, running, pausing, stopping and blocked states separately from Server job states. Keep preparation, speech, uploading and receipt reconciliation visible as phases, with elapsed time rather than invented completion percentages. Authentication/quota/configuration failures block repeated starts until their recovery condition is met; transient failures use bounded backoff.
- Implement a processing enabled/paused setting and bounded shutdown. Pausing stops new claims and lets the current attempt finish; quitting cancels/fences unfinished work through Server when reachable and terminates local children. Keep cancellation/retry operations in the runner without adding a job-management UI in v1. Retry uses a Server-authorized attempt of the same pinned generation; changed notes/profile/guide require a new generation through the appropriate API authority.
- On network loss, stop claims; continue active work only within the last confirmed lease's conservative validity bound. Stop it when validity is uncertain. On wake, reconcile before further work, discard stale outputs and recover any already committed receipt. Graceful quit stops claims, bounds cleanup time and releases claims when possible; forced exit relies on lease expiry and subsequent reconciliation. Resolve a cancellation/publication race from the Server receipt instead of displaying an unconfirmed cancellation as final.
- Enforce attempt and daily-start budgets in Server transactions across workers, restarts and retries. Count attempted launches consistently, including crashes around process creation. Pin the accounting period/timezone and report remaining allowance; a local cache or UI counter is never the authority. Test two Coach clients and changed machine clocks.
- Namespace cache entries by Server identity, job/generation, pinned profile/guide, adapter and media identity. Bound disk usage and clean only unreferenced local staging after reconciliation. Deleting all local cache must still permit Server-owned work to recover. Reconnect with stored receipts/IDs, never fabricated new submissions.

**Exit:** headless tests with the real Rust Server prove leases, pause/stop/quit, crash and receipt recovery without a GUI or real provider.

## C3. Implement the minimal v1 surface

Define behavior here; defer layout, visual styling and broader GUI design.

| Surface | Required behavior |
|---|---|
| Inbox drop area | Typed/pasted/dropped text and supported local text/audio attachments, explicit learning language, optional note and Send. Show selected material and validation errors before transmission. No URL fetching, folder traversal or source parsing. |
| Settings | Codex or Claude CLI selection, selected executable/runtime path and processing enabled/paused. Include only connection essentials (Server URL and scoped credentials) needed to run; keep operational limits and speech configuration out of a new settings dashboard. |
| Basic status | Connected/offline, selected CLI ready/missing/login required, idle/working/paused/error, and last submission accepted/pending/failed. A short actionable error and freshness timestamp are sufficient. |
| Database statistics | Server-reported phrase/word target counts, inbox counts by current status, and ready/study-only/awaiting-audio result counts. No raw SQL, database browsing, learner history or recording access. |

- Add a material-submit client using its own scoped capability, separate from worker and profile-update authority. Drag/drop or paste populates a draft; only Send transmits. Reject URL-only material and unsupported/oversized files, preserve supplied audio and require the text/language/role needed by the existing inbox contract. Never claim that local staging is accepted work.
- Persist a bounded pending-send envelope with stable submission ID and attachment hashes before transfer. A lost response or restart retries the exact same identity and bytes; clear the pending envelope only after durable Server acknowledgement or explicit abandonment. This is a sender outbox, not a second processing queue. Show unsent versus accepted distinctly; the Server owns processing after acknowledgement. Protect these temporary bytes and never read unrelated files or follow dropped symlinks outside selected input.
- Extend migration batch 4 with a bounded authorized summary endpoint, consistent count definitions and snapshot timestamp. Count logical current items, not attempts/revisions; count phrase/word targets by kind and current lifecycle. Result readiness categories are disjoint. Refresh after acknowledged submissions/publications and with bounded backoff; do not read Server SQLite directly. Offline/stale is not zero. Add contract tests for authorization, empty database, retries, retirement and mixed readiness.
- Persist the selected adapter as an enum, not an editable shell command. Apply selection changes to subsequent attempts; an active attempt retains its pinned adapter/model/configuration. Never rerun a completed attempt or switch to the other CLI because the selected one is missing, logged out or quota-limited. Missing the unselected CLI must not block submission or the selected adapter.
- Store versioned nonsecret machine settings atomically. Use macOS Keychain and Windows Credential Manager for scoped Server secrets through a tested adapter. Never copy provider tokens into settings, logs or job workspaces. Switching Server first drains/stops active work and preserves pending submissions under their original Server identity. Validate HTTPS; allow loopback HTTP only for development. Never disable certificate verification or send credentials across origin-changing redirects.
- Resolve tools from explicit configuration and documented discovery paths; show the selected executable. Do not assume shell initialization or PATH survives Finder/Start-menu launch. Support an explicit runtime when required instead of arbitrary shell strings or unverified wrappers.
- Keep input/settings accessible by keyboard with named roles, visible focus and readable errors. Prevent duplicate Send and accidental submission on drop. Sanitize failures before display; keep bounded logs of identifiers/phases/durations without raw prompts, audio or secrets. A log file is sufficient for v1; diagnostics export UI is deferred.

**Exit:** tests cover text/audio drops, explicit Send, validation, ambiguous acknowledgements, restart retry, CLI selection, unavailable tools, empty/stale statistics and basic processing status. Native keyboard/scaling acceptance remains in C5.

## C4. Integrate preparation and missing-reference speech

- Implement explicit Codex and Claude CLI adapters on the same typed runner interface. Use non-interactive execution (`codex exec`, Claude print mode), validate supported versions/flags during setup, and test each on Windows/macOS. Use the selected CLI's configured subscription login; no direct paid-API backend or fallback. Verify current launch/output/auth options against [Codex non-interactive documentation](https://learn.chatgpt.com/docs/non-interactive-mode) and [Claude programmatic execution](https://code.claude.com/docs/en/headless). Capability inspection does not authorize real model spending.
- Build on the [proven invocation contract](#proven-cli-invocation-contract): stage immutable input, guide, pinned profile, bounded candidate metadata and output schema in the per-attempt directory; pass only the bootstrap naming the prompt file inside it, with stdin null. Keep the adapter argument/environment builders as the only place that knows CLI flags.
- Pin adapter/version, model/effort configuration and request identity at attempt start. Keep prompt construction, provider argument building, process execution and result interpretation separate. Return one typed prepared-package result plus optional usage/timing and a classified failure. Normalize each CLI's structured result/envelope; progress text and a zero exit code are not a valid package. Reject empty, truncated, malformed, wrong-generation or schema-invalid output. Preserve unknown usage as unknown, not zero.
- Add parity fixtures for both adapters: long/multilingual prompt files, no TTY, spaces/quotes in paths, null stdin, progress mixed with final output, zero-exit/empty output, denied file access, cancellation and the recorded auth/quota outputs. Stdin delivery stays excluded rather than re-tested. Captured stderr can contain prompt text and command output, so keep it attempt-private; capture only sanitized metadata for reproducibility and do not store raw learner inputs in ordinary logs.
- Use argument arrays, prompt/input files, an allowlisted environment, isolated job directories, bounded simultaneous stdout/stderr draining, total byte/file limits and deadlines. Validate outputs before upload, including paths, symlinks and media sizes; no traversal outside staging. Distinguish malformed output, timeout, authentication, quota and cancellation. Keep secrets out of command lines.
- Run attempts through the existing supervisor: process groups with TERM-then-KILL on macOS, a kill-on-close Job Object assigned before the child resumes on Windows, bounded per-stream capture and descendant cleanup after normal exit. Add recovery for children that survive a killed Coach on macOS, where nothing like kill-on-close exists, and recheck Job Object compatibility with each CLI on Windows. Group termination is not a security sandbox. See [Microsoft Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) and [Tokio process lifecycle caveats](https://docs.rs/tokio/latest/tokio/process/index.html).
- Integrate the authentication/isolation approach of the [invocation contract](#proven-cli-invocation-contract) once the [C1](#c1-prove-the-desktop-boundary-on-both-operating-systems) gaps close. Recheck its permission cases against the complete adapter and repeat affected native probes when CLI versions, launch configuration or isolation change. Only pinned guide/input and bounded API context are available; learner recordings and unrelated files/accounts remain inaccessible. Fail the adapter acceptance gate on any regression; process cleanup remains a separate check.
- Rewrite the versioned preparation guide for current card/word/preference rules and bounded cross-entry matching. Unknown meaning and missing required preferences fail actionably. Treat imported text as data and learner notes as job-scoped instructions. Do not add generic LLM review APIs, source-specific parsers or routine publication approval.
- Integrate separately installed speech adapters. Pin exact speech input, engine/voice, locale/accent, settings and preparation version in cache identity. Use configured primary/fallback voices without changing language/locale, preserve supplied originals and never replace invalid supplied audio with TTS. Validate/transfer missing speech before practice readiness. Learner recordings never enter either provider path.
- Complete the synthetic submit → Coach offline → claim → prepare/synthesize → stage/upload → atomic publication → receipt → browser practice/replay workflow. Inject malformed outputs, missing audio, output overflow, process spawn failure, lease loss, cancellation during upload and connection loss after commit. Match UI status to Server receipts in every outcome.

**Exit:** both selectable CLI adapters and a speech configuration meet the platform/permission contract; fake-process tests cover every failure path with no real provider calls. Real semantic and listening acceptance follows migration batch 8 before unattended use.

## C5. Establish desktop and integration acceptance

- Cover runner transitions and error classification with deterministic clocks and fake boundaries; cover publication/recovery with the real Rust Server and disposable data. Headless tests must not require a window, operator credentials, `.env` or installed provider tools. Native fake executables must run on Windows and macOS; do not rely on POSIX scripts for Windows tests.
- Use [egui_kittest](https://docs.rs/egui_kittest/latest/egui_kittest/) to exercise real views through accessible roles/names, commands and observable status. Test long multilingual drop input, Send retries, both CLI selections, keyboard controls, fresh/stale counts and connection changes. Inject snapshots/events instead of sleeping to race background work. Keep UI fixtures and process/recovery tests with their responsibilities.
- Run native Windows/macOS smoke tests from packaged launch, including minimized processing, close/Quit, second-instance launch, sleep/wake, Server outage, killed children, disk-full/cache-write failure, denied credential-store access and missing tool updates. Test actual process trees and verify no owned descendants survive shutdown. Headless UI tests alone do not certify these behaviors.
- Verify VoiceOver/Narrator, text scaling, high DPI and both target macOS architectures; document exact tested OS/tool versions and remaining gaps. Publish no platform-support claim based solely on cross-compilation or a screenshot.
- Keep the migration's complete Rust/retained-client gate, Chromium and WebKit suites, including macOS browser recording coverage. Add Coach tests rather than replacing browser acceptance. Confirm Server alone serves practice, accepts inbox input and updates preferences with no GUI/provider installation.

**Exit:** native software conformance passes on the supported matrix; language quality, speech listening and actual learner-device acceptance remain separate and explicit.

## C6. Package, update and document the desktop app

- Package a macOS `.app` with stable bundle identifier, version, icons and resources, distributed in a DMG; build Apple Silicon and Intel artifacts, combining into a universal app only after both pass. Package a Windows GUI-subsystem executable with version/icon resources and a per-user installer plus a portable ZIP for diagnostics. Choose and pin installer tooling in this packaging batch; use the same artifact inputs in CI and local builds. Keep Server and Vue assets in their separate deployment artifact.
- Keep agent CLIs, speech tools and model/voice assets separately installed and documented with compatible versions and licenses. The Coach artifact must start without them and show setup actions. It must not require Rust, Node or the project's Python on the end user's machine; external tools may have their own explicit runtime requirements.
- Build/package on native CI runners using the locked Rust/Node gate, including Coach's desktop feature. Add these jobs alongside the existing Python checks during transition; batch 7 removes the obsolete jobs after replacement acceptance. Smoke-test extracted/installed artifacts rather than only `cargo run`: resource paths, config migration, credential lookup, fake process launch, drop submission and summary counts. Test install, replacement/upgrade, interrupted update and uninstall while preserving Server-owned work; never write config/cache beside installed binaries.
- Support manual application replacement for this release. Quit and reconcile before replacement; check API/package compatibility at startup and block incompatible claims with a clear upgrade action. Local config migration must preserve secrets and permit a documented recovery path. Coach updates never migrate Server's database or mutate accepted jobs locally.
- Prepare signing as an explicit distribution gate: Developer ID, hardened runtime and notarization/stapling for the macOS distribution; Authenticode signing for Windows artifacts. Test external-tool launch from the signed app. Treat unsigned/ad-hoc artifacts as development outputs. Follow [Apple's notarization guidance](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution); actual signing credentials, uploads and distribution require the relevant release authorization.
- Write installation/setup and troubleshooting documentation for both OS families: Server credential provisioning, tool login outside ordinary jobs, missing preferences, run budgets, pause versus stop, sleep/reconnect, quit, stale leases, pending delivery, manual update and sanitized log location. Measure idle Coach resource use separately from agent/speech processes and Server. Keep the app useful when offline by explaining recoverable state, not by inventing a local queue authority.

**Exit:** versioned desktop artifacts built from the transition tree install and recover on the claimed platforms, operational docs match their behavior and the applicable transition gates pass. This completes packaging acceptance before cutover; [migration batch 7](implementation-plan.md#7-cut-over-and-remove-every-python-scaffold-dependency) separately rebuilds and verifies artifacts after Python removal. Signing/distribution acceptance and private deployment remain separately authorized actions.

## Ideas adopted from the reference repository

Read `../ibkr-porez-rs/` only. Its `Cargo.toml`, `src/bin/gui.rs`, `src/gui/app.rs`, `tests/test_e2e_gui.rs` and `.github/workflows/release.yml` inform these choices:

| Idea | Phrase Gym adaptation |
|---|---|
| egui/eframe and optional GUI dependencies | Suitable for drop input/status/settings; validate current versions and accessibility, and isolate them from Server. |
| Background results and UI repaint requests | Use typed bounded channels, one runner and explicit cancellation/lease state. Keep native UI effects on the appropriate thread. |
| egui_kittest view interaction | Reuse the testing approach with injected state and observable behavior, without network work during rendering. |
| Platform icons, Windows subsystem and macOS bundles | Apply to Coach packaging; choose a simple per-user install, without the reference app's privileged CLI symlink installer. |
| Native CI matrix and release optimization | Measure size/startup/idle behavior; pin the toolchain and build artifacts before any release request. |

Do not copy its tax workflow, file-backed authority, broad application controller, automatic sync schedule, dependency versions, ad-hoc signing as release acceptance or automatic tag/upload recipes. Coach coordinates durable Server jobs and long-lived external processes; that requires its own state and recovery boundaries.

## Proven CLI invocation contract

Verified on 2026-10-17 with Claude Code 2.1.302 and codex-cli 0.159.2 on macOS 27.0.1 (Apple Silicon), using synthetic canaries, Claude `haiku` and Codex `gpt-5.6-luna` at low effort; results are in the [setup notes](../../docs/coach-desktop.md#agent-cli-setup). `phrase-gym-coach` encodes it in `adapter::{claude,codex,environment}` with unit tests; `examples/cli_probe.rs` repeats the real probes. Re-run them when a CLI version or any flag changes. The prompt-file, stdin, `--`, `--bare`, Claude tool and environment findings below come from `news-recap` and are settled, not to be rediscovered.

**Shared rules.**

- *Executable:* resolve the canonical path (symlinks followed) from the documented install locations (`~/.local/bin`, `/opt/homebrew/bin`, `/usr/local/bin`, `%USERPROFILE%\.local\bin`) and the GUI PATH; never rely on shell initialisation. Claude's native installer keeps only the two newest versions, so recheck the path before each attempt.
- *Attempt layout:* a fresh directory per attempt under the Coach cache holds `prompt.md`, the staged inputs and bounded `stdout.log`/`stderr.log`; it is also the working directory, so no repository instructions or git context load.
- *Prompt delivery:* the full prompt stays in the file; the only argument is `Read your task from <absolute prompt path> and execute it.`; stdin is null. Never stdin: `claude -p` returns empty output once piped stdin exceeds a few KB ([anthropics/claude-code#7263](https://github.com/anthropics/claude-code/issues/7263)), and `codex exec` with a non-TTY stdin hangs on EOF ([openai/codex#20919](https://github.com/openai/codex/issues/20919)); with null stdin Codex prints `Reading additional input from stdin...` and continues.
- *Environment:* built from an allowlist, nothing inherited. macOS: `HOME USER LOGNAME SHELL TMPDIR LANG LC_ALL LC_CTYPE`; Windows: system/profile folders, `TEMP`/`TMP`, user/computer names and processor variables. `PATH` is the executable's directory plus system directories. Everything else is dropped, including `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`, `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `CODEX_API_KEY`, proxies and parent agent session variables (`CLAUDECODE`, `CLAUDE_CODE_*`). With `ANTHROPIC_API_KEY` present `claude auth status` reports API-key auth; with the allowlist it reports the claude.ai subscription.
- *Supervision:* per-attempt deadline; deadline, cancellation or output overflow sends TERM to the process group and KILL after a grace period (Windows terminates the Job Object); descendants are killed after a normal exit too.
- *Results and failures:* exit 0 with empty stdout is a failure ("no output produced"). Readiness checks without model usage: `claude auth status` must show `loggedIn: true` and `authMethod: "claude.ai"`; `codex login status` must print `Logged in using ChatGPT`. Recorded outputs: Claude prints `Not logged in · Please run /login` to stdout and exits 1; Codex without a login retries five times, then prints `unexpected status 401 Unauthorized`; Codex refusing an API-key login prints `ChatGPT login is required, but an API key is currently being used. Logging out.` Quota/rate detection reuses `news-recap` patterns (`rate.?limit`, `too many requests`, `\b(status|code|http)\b\W{0,10}429` (a bare `429` also matches PIDs and trace ids), `credit balance`, `Individual quota reached|upgrade your subscription`, `overloaded`) over stderr and stdout.
- *Output channel:* plain stdout is the default and proven for both CLIs. Claude `--output-format json` adds `result`, `is_error`, `permission_denials` and usage, and is the candidate for C4 normalisation; Claude text mode reports no usage. Codex prints `tokens used\n<total>` to stderr; `--json` adds per-turn input/cached/output/reasoning tokens; `--output-last-message <file>` only duplicates the final stdout message.

**Claude.**

```text
<claude> -p --model <model> [--effort <level>] --safe-mode --restricted --strict-mcp-config
  --tools Read --allowed-tools Read --permission-prompts none --no-session-persistence
  --disable-slash-commands -- "Read your task from <prompt> and execute it."
env += CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 ENABLE_CLAUDEAI_MCP_SERVERS=false
```

- `--restricted` ignores user/project/local settings (permission allow lists, enabled plugins) and the user's `CLAUDE.md`, removes code-running tools and confines file tools to the working directory: reading a canary or `~/.claude.json` outside it fails with "--restricted confines the file tools to the working directory". `--tools Read` alone still loads the user's `CLAUDE.md`; `--safe-mode` alone still offers Bash, WebFetch and Write. Together with `--strict-mcp-config` the session init lists only the Read tool, no MCP servers, skills or slash commands, two built-in plugins and `apiKeySource: none`.
- `--` is required because `--allowed-tools` is variadic. `--allowed-tools Read` lets Read run without a prompt; `--permission-prompts none` denies anything that would prompt. The environment additions stop background self-updates and telemetry during a pinned attempt and the fetching of claude.ai connectors.
- `haiku` resolves to Claude Haiku 5.5, whose thinking `MAX_THINKING_TOKENS=0` cannot disable; `CLAUDE_CODE_MAX_OUTPUT_TOKENS=64000` remains the large-output knob. Never set `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING` alone; it pushes reasoning into stdout on models that honour it.
- Never: `--bare` (limits auth to `ANTHROPIC_API_KEY`), `--permission-mode dontAsk|bypassPermissions`, `--dangerously-skip-permissions`, WebFetch/Bash tools or `--add-dir`.

**Codex.**

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

- A permission profile replaces `--sandbox workspace-write`: that legacy sandbox lets commands read the whole disk and write `/tmp`, and passing any `--sandbox` makes Codex ignore permission profiles. `codex sandbox -P` showed `:minimal` opens `/tmp` and `$TMPDIR` writes unless denied explicitly, and that the sandbox helper re-executes Codex by its invoked path, so the install directory needs read access and Codex must be launched by its canonical path (a rule on the Homebrew symlink is not enough). With the profile, canary reads/writes outside the attempt and `~/.codex/auth.json` fail with "Operation not permitted" and `curl` cannot resolve hosts.
- `--ignore-user-config` drops `config.toml` (MCP servers, plugins, `notify`, model, developer instructions) but still loads `$CODEX_HOME/AGENTS.md` and `$CODEX_HOME/skills`, shown by capturing the request of a synthetic home against a local endpoint. Coach therefore owns a separate Codex home with its own ChatGPT login. Bundled system skills (imagegen, openai-docs, skill-creator, skill-installer) still appear; the remaining tools are command execution, stdin writes, patching, waiting and user-input requests. `view_image` (reads files from the unsandboxed Codex process) and goals are disabled.
- `forced_login_method="chatgpt"` refuses and logs out an API-key login; without it a stored API key is used. `--skip-git-repo-check` is required outside a git repository. An unknown `--disable` name fails the launch, which surfaces version drift; unknown `-c` keys are ignored with a stderr warning.

**Do not copy** from `news-recap`: Antigravity, direct-API execution, news task routing, arbitrary web-fetch tools, permission-bypass flags, broad command templates with network access, copying the parent environment, project-root execution or terminating only the immediate child.
