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

The local server binds loopback and rejects cross-site mutations. This is a local development application; permanent private hosting/access policy, broader device acceptance and verified backups remain release work. Provider keys stay server-side. No LLM or speech-provider requests are needed to browse prepared content.

## Project context

Read [specs/START-HERE.md](specs/START-HERE.md) for current decisions and [the implementation plan](specs/plan/implementation-plan.md) for remaining work. Specs retain design rationale and constraints; code, tests and generated API contracts describe implemented behavior. User documentation remains under `docs/`.

[Documentation](https://andgineer.github.io/phrase-gym/) · [License](LICENSE)
