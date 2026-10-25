[![Build Status](https://github.com/andgineer/phrase-gym/workflows/CI/badge.svg)](https://github.com/andgineer/phrase-gym/actions)

# Phrase Gym

A language trainer for ready use of words, phrases and prepared dialogues.

Phrase Gym Server (`crates/phrase-gym-server`) serves the Vue client in `frontend/` and the HTTP API: persistent language settings and personal preferences, a searchable library, reference audio, unified practice sessions, optional durable recordings, prepared dialogue rehearsal and FSRS scheduling. Phrase Gym Coach (`crates/phrase-gym-coach`), a Windows/macOS desktop app, prepares material and reference speech through the Server's durable inbox. Both are Rust.

## Requirements

- Rust through rustup; `rust-toolchain.toml` pins the exact toolchain.
- Node 22.12+ on an LTS release (`.nvmrc`).
- For Coach preparation: the Codex or Claude CLI with its subscription login and a separately installed Piper runtime, as described in the [Coach setup](docs/coach-desktop.md).

## Run locally

```bash
npm --prefix frontend ci
npm --prefix frontend run build
cargo xtask run --data-dir /private/tmp/phrase-gym-preview
```

Open [the local app](http://127.0.0.1:8000/learn/en). Choose your native language on first launch. English, Serbian, German and Russian UI locales are available; explanations stay in the native language. The database starts without content. `/learn/en` and `/learn/sr` can be bookmarked independently. Retain the data directory between restarts. Browser journals are pending-transfer storage, not backups.

Server and Coach run directly on the same MacBook at `http://127.0.0.1:8000`, without connection setup or application authentication; follow the [preview access and launch instructions](docs/rust-preview.md). The global **+** opens a clipboard preview over any screen: review or edit the phrase, add an optional note, then explicitly **Send**. No manual lesson fields are required. Coach also accepts deliberate raw phrase paste automatically; both paths durably acknowledge original text before Codex or Claude preparation and generated Piper reference speech, then publish atomically. External agents can submit prepared packages and original audio through the [inbox API](docs/preview-inbox.md). Normal **Start session** opens new English material as ungraded study with **Next**, reference playback and retained recordings. After completed study, a later Start offers independent recall with the answer hidden until reveal and honest self-rating afterward. Resume keeps the existing session. Recording is optional, and every take is retained automatically. See the [Coach demo](docs/coach-desktop.md); full multilingual/content/desktop acceptance remains in the implementation plan.

Phrase Gym Coach is not released yet; [its setup notes](docs/coach-desktop.md) cover the development build, supported OS versions, agent CLI login, Piper speech and the manual acceptance checklist.

The Server has local `backup`, `verify-backup` and `restore` commands for consistent SQLite-plus-media snapshots and isolated recovery. Follow the [backup and rollback instructions](docs/recovery.md) before upgrading a data directory.

The local Server binds loopback and rejects cross-site mutations. This is a local development application; permanent private hosting/access policy, broader device acceptance and operational recovery rehearsals remain release work. The Server holds no provider credentials, and browsing prepared content needs no LLM or speech-provider requests.

## Checks and build

```bash
cargo xtask install-hooks # once per clone: commits then run `cargo xtask pre` first
cargo xtask pre           # rustfmt, Clippy, architecture and hygiene checks, client formatting, generated-contract drift
cargo xtask test          # Rust tests, client typecheck/unit/build, Chromium and WebKit suites
cargo xtask api-types     # refresh frontend/openapi.json and frontend/src/api.generated.ts from the Server
cargo xtask app-build     # build the client and release Rust binaries
cargo xtask coach-bundle  # unsigned development bundle of the Coach desktop app (macOS or Windows)
cargo xtask coach-smoke   # launch that bundle like a user and verify its window, process supervision and failure paths
```

Install the browser binaries once from `frontend/` with `npx playwright install chromium webkit`. Builds with `--features test-support` add content-seeding routes under `/api/v1/imports` and deterministic browser fault controls under `/_test`; release builds and the OpenAPI document omit them. The browser suite starts disposable Servers and runs the built Vue client in Chromium and WebKit. `fixtures/preservation/` holds language-neutral HTTP scenarios and recorded scheduler sequences that encode rules the Server keeps.

## Documentation

- [Preview access and launch](docs/rust-preview.md)
- [Coach desktop setup, Piper speech and measurements](docs/coach-desktop.md)
- [Inbox API for external agents](docs/preview-inbox.md)
- [Backup, restore and rollback](docs/recovery.md)

## Project context

Read [specs/START-HERE.md](specs/START-HERE.md) for current decisions and [the implementation plan](specs/plan/implementation-plan.md) for remaining work. Specs retain design rationale and constraints; code, tests and generated API contracts describe implemented behavior. Development rules live in [AGENTS.md](AGENTS.md).

[License](LICENSE)
