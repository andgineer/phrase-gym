[![Build Status](https://github.com/andgineer/phrase-gym/workflows/CI/badge.svg)](https://github.com/andgineer/phrase-gym/actions)

# Phrase Gym

A language trainer for ready use of words, phrases and prepared dialogues.

The production app in `src/phrase_gym` and `frontend/` includes persistent language settings, a searchable library, reviewed manual-package publication, reference audio, unified practice sessions, optional durable recordings, prepared dialogue rehearsal and FSRS scheduling.

## Run locally

Use Python 3.12+ with uv and Node 22.12+.

```bash
uv sync --frozen
cd frontend
npm ci
npm run typecheck
npm run build
cd ..
uv run phrase-gym --data-dir .data serve --client-dir frontend/dist
```

Open [the local app](http://127.0.0.1:8000/learn/en). Choose your native language on first launch. English, Serbian, German and Russian UI locales are available; explanations stay in the native language. The database starts without content. `/learn/en` and `/learn/sr` can be bookmarked independently.

For sample content, validate and stage a package:

```bash
uv run phrase-gym --data-dir .data import-package fixtures/manual-en.json
uv run phrase-gym --data-dir .data import-package fixtures/manual-en.json --stage
# Inspect the staged report and material, then use the returned import ID:
uv run phrase-gym --data-dir .data publish IMPORT_ID --reviewed
```

Repeat for `fixtures/manual-sr.json`. These are structural examples, not a release content pack; they deliberately contain no production audio. Reference playback requires an exact full-phrase clip imported separately before publication:

```bash
uv run phrase-gym --data-dir .data upload-reference phrase.wav metadata.json --mime-type audio/wav
uv run phrase-gym --data-dir .data reconcile-media
```

Use the returned media ID in the package. Metadata declares language, exact text, role, origin and provenance; generated speech also identifies its engine, voice and speech input. Inspect the actual schema at `/docs`. Reconciliation reports missing/corrupt references and orphan files while retaining originals.

Start session drills due/new targets in the selected scope; Resume keeps the existing session. Recording is optional, and every take is retained automatically. Keep a tab open when audio storage fails and use Retry; do not clear browser storage to upgrade. Retain the data directory between restarts. Browser journals are pending-transfer storage, not backups.

## Checks and build

```bash
uv run pytest tests/
uv run pyrefly check --project-excludes=tests --python-interpreter-path=.venv/bin/python
cd frontend
npm run typecheck
npm test
npm run build
npx playwright install chromium webkit
npm run test:browser
cd ..
./build.sh
```

`build.sh` bundles the client in the Python wheel/source distribution. A backend-only `uv build` before preparing the client requires `--client-dir` at runtime. Invoke tasks include `app-check`, `app-build`, `api-types`, and the scaffold's existing documentation/version tasks. Run `uv run invoke --list` or `uv run phrase-gym --help`.

The Rust replacement lives in the Cargo workspace (`crates/`, `xtask/`) and runs alongside the Python application during the migration. With rustup and Node 22.12+ installed:

```bash
cargo xtask pre        # rustfmt, Clippy, architecture and hygiene checks, client formatting, generated-contract drift
cargo xtask test       # Rust tests, client typecheck/unit/build, Chromium and WebKit suites
cargo xtask api-types  # refresh frontend/tests/rust-contract from the Rust OpenAPI document
cargo xtask app-build  # build the client and release Rust binaries
cargo xtask run  # Rust server on 127.0.0.1:8000 with synthetic data
cargo xtask coach-bundle  # unsigned development bundle of the Coach desktop app (macOS or Windows)
cargo xtask coach-smoke   # launch that bundle like a user and verify its window, process supervision and failure paths
```

The Rust demo runs Server and Coach directly on the same MacBook at `http://127.0.0.1:8000`, without connection setup or application authentication; follow [preview access and launch instructions](docs/rust-preview.md). The production [bounded English lesson inbox](docs/preview-inbox.md) supports durable supplied-audio intake and fake-worker publication; the Library’s **Add a first lesson** action accepts one English phrase with its original clip and shows durable delivery/preparation status. Choosing, dropping or pasting the original clip starts delivery automatically; normal **Start session** opens published English material as ungraded study with **Next**, reference playback and retained recordings. Claude preparation and native Coach integration remain preview work.

Phrase Gym Coach, the Windows/macOS preparation app, is not released yet; [its setup notes](docs/coach-desktop.md) cover the development build, supported OS versions, agent CLI login and the manual acceptance checklist.

The Rust Server has local `backup`, `verify-backup` and `restore` commands for consistent SQLite-plus-media snapshots and isolated recovery. Follow the [backup and rollback instructions](docs/recovery.md) before upgrading a Rust data directory.

The Rust server implements every endpoint the retained client calls; the Python preparation and handoff routes are not ported because the inbox replaces them. It uses its own schema and does not open databases created by the Python application. Builds with `--features test-support` add content-seeding routes under `/api/v1/imports` and deterministic browser fault controls under `/_test`; release builds and the OpenAPI document omit them. The browser suite starts disposable Rust servers and runs the built Vue client in Chromium and WebKit. Only the named `*-python-preparation` projects use the temporary Python preparation server until that workflow moves to Rust. Preservation fixtures under `fixtures/preservation/` record the Python behavior that encodes rules the Rust server keeps.

The local server binds loopback and rejects cross-site mutations. This is a local development application; permanent private hosting/access policy, broader device acceptance and operational recovery rehearsals remain release work. Provider keys stay server-side. No LLM or speech-provider requests are needed to browse prepared content.

## Project context

Read [specs/START-HERE.md](specs/START-HERE.md) for current decisions and [the implementation plan](specs/plan/implementation-plan.md) for remaining work. Specs retain design rationale and constraints; code, tests and generated API contracts describe implemented behavior. User documentation remains under `docs/`.

[Documentation](https://andgineer.github.io/phrase-gym/) · [License](LICENSE)
