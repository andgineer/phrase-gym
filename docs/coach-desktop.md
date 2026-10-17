# Phrase Gym Coach desktop: development setup

Coach is not released. This page covers the development build, the platform facts measured so far, how to prepare the agent CLIs and what still has to be checked by a person. Product behavior lives in the [Coach decision](../specs/decision-external-preparation-agent.md); remaining work lives in the [Coach plan](../specs/plan/coach-desktop.md).

## Build, run and smoke-test

| Task | Command |
|---|---|
| Run the status window from the checkout | `cargo run --package phrase-gym-coach --features desktop` |
| Build an unsigned development bundle | `cargo xtask coach-bundle [--out DIR]` |
| Build, launch like a user and verify the bundle | `cargo xtask coach-smoke` |

`coach-bundle` writes `Phrase Gym Coach.app` on macOS (the fake CLI used by the checks sits in `Contents/Resources`) and `Phrase Gym Coach\phrase-gym-coach.exe` with a `resources` folder on Windows. It is a development layout: unsigned, no icon, no installer.

`coach-smoke` places the bundle under a path with spaces and non-ASCII characters and starts it the way a person does. On macOS that is LaunchServices (`open`) with only the variables launchd gives a Finder launch, because `open` forwards its own environment. On Windows it is `Start-Process -UseNewEnvironment` from PowerShell. The app then writes a report, and the smoke test fails unless all of these hold:

- the real window opened on a hardware or software renderer;
- at most 2 frames were drawn during a 3-second idle period;
- the bundled fake CLI received Unicode arguments and its working directory, saw none of the launching terminal's variables, was cut off at the 64 KiB output limit and left no grandchild running;
- the per-user folders exist and the credential store answered a read-only probe.

The report also lists the Codex and Claude executables found without the shell's PATH and their `--version` output. Two failure paths are checked as well: a launch with no usable renderer (`WGPU_BACKEND=gl`) must exit with code 2 and an actionable message, and a launch while Coach's instance lock is held must exit with code 3. CI runs the smoke test on macOS Apple Silicon, macOS Intel and Windows x64.

## Supported platforms

| Component | macOS | Windows |
|---|---|---|
| Coach runtime: Rust 1.96, eframe/winit, wgpu with Metal or Direct3D 12 | 11+ on Apple Silicon (Rust target minimum); Metal needs 10.13+ on Intel | 10+ x64 (Rust target minimum, Direct3D 12) |
| [Claude Code](https://code.claude.com/docs/en/setup) | 13.0+ | 10 version 1809+ or Server 2019+ |
| [Codex CLI](https://github.com/openai/codex/blob/main/docs/install.md) | 12+ | officially Windows 11 via WSL2; a native PowerShell installer also exists |
| **Coach claims** | **13.0+ on Apple Silicon and Intel** (`LSMinimumSystemVersion` 13.0) | **10 version 1809+ x64**, with the Codex gap below |

A supported release needs both CLI adapters, so the macOS floor follows Claude Code. Native Windows Codex is unverified: its own documentation names WSL2, and Coach launches native executables only. Tested so far: macOS 27.0.1 on Apple Silicon by hand, plus the CI images. The oldest claimed versions, macOS 13 and Windows 10 1809, are untested.

## Toolkit and measurements

egui, eframe and egui_kittest 0.37.0 with wgpu 30.0.1. Only the Metal and Direct3D 12 backends are compiled. AccessKit comes in through eframe's `accesskit` feature, and the status window prefers the low-power GPU. Measured on an Apple M1 Pro with macOS 27.0.1 and the release bundle:

| Measurement | Result |
|---|---|
| Idle CPU after startup | 0.0%: 0.06 s of CPU time in 60 s, 0 idle wake-ups |
| Redraws while idle | 2 frames in a 3-second window, 10 frames for the whole smoke run |
| Memory | about 100 MB resident (renderer included) |
| Size | 9.4 MiB release binary; 9.8 MiB development bundle including the 0.45 MiB fake CLI |

Renderer fallback: none was added. The local window starts on Metal, and each CI smoke report records the renderer its runner found. If the renderer cannot start, a native dialog explains the next step: update the graphics driver, or enable graphics acceleration in a Remote Desktop or virtual machine session.

## Per-user folders and credentials

| Purpose | macOS | Windows |
|---|---|---|
| Settings and instance lock | `~/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach` | `%APPDATA%\andgineer\Phrase Gym Coach\config` |
| Cache and attempt staging | `~/Library/Caches/io.github.andgineer.Phrase-Gym-Coach` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\cache` |
| Logs | `~/Library/Logs/Phrase Gym Coach` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\data\logs` |
| Coach's own Codex home | `~/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/codex-home` | `%LOCALAPPDATA%\andgineer\Phrase Gym Coach\data\codex-home` |

Secrets go only to the login Keychain or Windows Credential Manager, under the service `io.github.andgineer.phrase-gym-coach`. At startup Coach only reads that store. If access is denied, the status window says so, and Coach never saves the secret anywhere else. A second Coach for the same user refuses to start and says one is already running.

## Agent CLI setup

Verified with Claude Code 2.1.302 and codex-cli 0.159.2 on macOS 27.0.1 (Apple Silicon) on 2026-10-17. The exact invocation and the evidence behind each flag are in the [Coach plan](../specs/plan/coach-desktop.md#proven-cli-invocation-contract).

**Claude CLI** uses the existing subscription login. Check it without using the model: `claude auth status` must report `"loggedIn": true` and `"authMethod": "claude.ai"`. Coach strips `ANTHROPIC_API_KEY` and similar variables, so an exported key never takes over.

**Codex CLI** gets a separate Codex home that belongs to Coach, so the instructions (`AGENTS.md`) and skills in your normal `~/.codex` never reach a job. Sign in to it once with your ChatGPT plan:

```bash
CODEX_HOME="$HOME/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/codex-home" codex login
CODEX_HOME="$HOME/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/codex-home" codex login status
```

The status must read `Logged in using ChatGPT`. Coach forces ChatGPT login, so an API-key login in that home is refused and logged out instead of being billed.

What a job can and cannot do, checked with synthetic canary files and the cheapest models (Claude `haiku`, Codex `gpt-5.6-luna` at low effort):

| Check | Claude CLI | Codex CLI |
|---|---|---|
| Read the staged input in the attempt folder | allowed, exact text returned | allowed, exact text returned |
| Read a canary file outside the attempt folder | denied by the CLI ("confines the file tools to the working directory") | denied by the sandbox ("Operation not permitted") |
| Write outside the attempt folder | impossible: Read is the only tool | denied by the sandbox; nothing written |
| Web access | no web tool | web search disabled; `curl` cannot resolve hosts |
| Read the CLI's own credential/config files | `~/.claude.json` denied | `~/.codex/auth.json` denied |
| User instructions, MCP servers, plugins, hooks, connectors | none loaded: only the Read tool, no MCP servers, skills or slash commands | none from `config.toml`; tools limited to command execution and patching. Bundled system skills are still listed |
| Launch from a LaunchServices-started app without the shell's PATH | works | works |

Each real call took 3 to 12 seconds. A trivial Codex attempt reports about 8,000 tokens; Claude reports no usage in text mode.

## Remaining gaps

- **Codex home login.** The probes used the operator's own `~/.codex` with user configuration ignored. A run inside Coach's own Codex home needs your one-time login, followed by the confirming probe in the checklist below.
- **Windows CLIs.** No Windows probes have run with logged-in CLIs. Still unverified there: native Codex support and whether its permission profile is enforced, Claude's `--restricted` confinement, and where the tools are installed. CI covers only the build, the UI tests, process cleanup through the Job Object, the credential store and the packaged smoke test.
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
9. **Codex in Coach's home.** After the one-time login above, run `cargo run --package phrase-gym-coach --example cli_probe -- codex gpt-5.6-luna io "<scratch folder>" "<Coach codex-home>"`. It must print the staged line, `canary leaked: false` and `outside file written: false`. Repeat with the `outside-read` and `credentials` probes.
10. **Windows CLIs.** On a Windows machine with both CLIs logged in, run the probe example for `io`, `outside-read`, `outside-write`, `web` and `credentials` for each CLI.
