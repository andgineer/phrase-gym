# P2 foundation — run and extend

Implemented 2026-10-12. This is the production foundation, not the P3 trainer or a deployed release. The user selected `frontend/` for the Vue/TypeScript client. FastAPI/Pydantic, SQLAlchemy/Alembic and SQLite extend the existing Python scaffold.

## Local runtime

Python 3.12+ and Node 22.12+ are required. Install locked dependencies with `uv sync --frozen` and `npm --prefix frontend ci`. Build/check the client separately:

```bash
npm --prefix frontend run typecheck
npm --prefix frontend run build
uv run phrase-gym --data-dir .data serve --client-dir frontend/dist
```

Open `http://127.0.0.1:8000/learn/en`. For hot client development, run `npm --prefix frontend run dev` in a second terminal and use its loopback Vite URL. Its `/api` proxy uses the same-origin backend API. The server's mutation guard accepts matching browser origins; it does not enable wildcard CORS.

Runtime configuration is explicit:

| Setting | Behaviour |
|---|---|
| `--data-dir` / `PHRASE_GYM_DATA_DIR` | SQLite, hash-addressed original reference files and upload staging. Default is the OS user application-data directory. Never the source tree unless explicitly selected. |
| `--client-dir` / `PHRASE_GYM_CLIENT_DIR` | Built static client; otherwise uses a bundled client when present. |
| `--port` | Loopback API/client port, default 8000. |
| `PHRASE_GYM_HOSTS` | Trusted Host names, default `127.0.0.1,localhost`. |
| `PHRASE_GYM_ORIGINS` | Optional exact allowed mutation origins for a later explicitly configured proxy. Empty by default. This is not authentication. |

`serve` binds loopback and disables forwarded-header trust. It has no public account or production authentication implementation. Private HTTPS access, iPhone/PWA use and verified backups remain later work. No provider environment values are returned to the client; `.env` is ignored and no provider calls occur.

Migrations run before the server starts serving requests. SQLite uses foreign keys, WAL and a busy timeout. Domain writes acquire `BEGIN IMMEDIATE` to serialize publication/check-and-write operations. Domain services accept an injected clock and have no dependency on FastAPI.

## Manual content

`fixtures/manual-en.json` covers fixed phrase, word sense, frame, shared lesson membership and a prepared dialogue. `fixtures/manual-sr.json` covers Serbian text and native Russian explanations. They are structural/content examples with manually declared cue review, not a certified release pack or audio-quality evidence. They do not install themselves or change user data automatically.

```bash
uv run phrase-gym --data-dir .data import-package fixtures/manual-en.json
uv run phrase-gym --data-dir .data import-package fixtures/manual-en.json --stage
uv run phrase-gym --data-dir .data publish IMPORT_ID --reviewed
```

Dry-run is the default. Staging writes only a draft and report. Explicit publication rechecks media and the namespace's content version under the write lock; stale staging returns a conflict. Retrying publication returns the same receipt. The CLI's `--reviewed` and API's `{"reviewed": true}` attest to manual content review; structural validation alone is not linguistic review.

Stable `(namespace, source_id)` references map to internal UUID targets. Reordering, lesson renaming and reimport preserve those identities. A changed language, kind, meaning key or required expression under an existing source ID is rejected; give a new learning objective a new source ID. Omitted items are not implicitly deleted. Cross-source merging requires a future explicit reconciliation flow; there is no text-similarity merge.

The immutable target-revision JSON holds the initial realization, chunks, checked cues, acceptable equivalents, frame slots, explanations and concept metadata. These are not independently scheduled cards. Dialogue revisions store target UUIDs **and pinned target revision numbers**. Updating referenced target content publishes a new dialogue binding while retaining the old one. P3 can add sessions and schedules against these identities without reshaping them into sibling cards.

APIs are under `/api/v1`; inspect `/docs` for the contract. Reads include settings, paginated lessons, phrase search, phrase detail (optional historical revision), complete lesson phrase lists, and paginated/versioned dialogues. Searches use Unicode casefolded substring matching over phrase text, purpose and manually prepared concepts. Lesson title search is independent of phrase lookup. All content reads require a supported learning language and reject foreign-language IDs. Initial locales are `en`, `sr`, `de`, `ru`.

The client uses `/learn/{language}` plus language-preserving phrase/lesson routes. Explicit routes win over the last-language preference in every tab. A bare route restores the last language; invalid routes show selection and supported empty languages show an empty library. UI follows native or learning language; missing native explanations are explicit rather than falling back to the UI language. Back navigation retains the original search and complete lesson list. Start session/drill controls remain disabled with an explanation until P3.

Search refreshes keep inputs mounted and distinguish failed requests from empty results. Pagination failures retain loaded lesson/phrase lists and permit retry; stale query/page responses cannot change a newer route. The [accepted transition checklist](p2-review-convergence.md) defines the pending/success/failure/cancellation behavior and its coverage.

Initial settings loads share one read across route changes. Preference mutations use one per-tab ordered queue; pending language updates retain the latest choice so a delayed older response cannot change bare-route restoration after acknowledgement. Native/UI settings saves retain submission order. Their full responses preserve a language remembered by navigation after submission, keeping the tab cache and server preference coherent through history and bare-route restoration. Other tabs keep their own explicit language routes. Settings controls' accessible names follow the effective UI language.

Reference lookup logs an idempotent exposure with server receipt time and no recall grade. First-launch/configuration-hidden materials create no exposure. Each pending event has its own localStorage key, retained before revealing the materials and removed only after server acknowledgement. Saving runs independently of display and navigation; pending writes show a saving status, failed writes offer retry. Reload retries failed exposure writes using the same event ID. This small reference-event outbox is not the future learner-audio store: P3 must use IndexedDB for every pending take and session checkpoint, and persist recordings as SQLite BLOBs.

## Reference audio

Upload the original full-phrase file and metadata before attaching its returned UUID in a package:

```bash
uv run phrase-gym --data-dir .data upload-reference phrase.wav metadata.json --mime-type audio/wav
```

Example metadata (replace with real reviewed provenance):

```json
{
  "language": "en",
  "text": "How has your day been so far?",
  "role": "reference",
  "origin": "course",
  "source": "Stable course/clip identifier",
  "performer": "Actual performer credit",
  "locale": "en-US",
  "accent": "US",
  "preparation_version": "source-original-1"
}
```

Supported containers are WAV, MP3, Vorbis Ogg and audio MP4/M4A, up to 12 MiB. WAV validation checks complete PCM frame bytes; other formats use container/duration inspection. Real speech decoding/listening and device acceptance remain open. Browser walkthrough audio is an explicitly labelled generated tone, never a speech-quality or actor-audio pass.

Media identity includes the bytes hash, MIME type and provenance. Upload retries reuse that identity; changed performer/voice/preparation settings cannot silently reuse a metadata identity. File content is addressed by SHA-256 and original bytes are preserved. Generated-audio metadata requires engine/voice/exact speech input; generation adapters themselves remain P5 work. Use the echo-words Piper/Edge/llmbroker guidance then, not macOS voices.

Publish validates exact text, language and role for reference/partner clips. Cue audio text must equal precisely the ordered visible chunks; a full-answer or headword-only clip cannot be substituted. A target without its full reference remains audio-missing/not retrieval-ready. Browser playback begins on a gesture and stops on navigation/background. Missing or damaged files return a recovery error, never another language's fallback.

Files are staged, flushed and atomically moved before metadata commit. A crash may leave an orphan hash, which is reported and reusable on retry. Reconcile with:

```bash
uv run phrase-gym --data-dir .data reconcile-media
```

Reconciliation reports missing/corrupt metadata references and orphan hashes, retaining originals; only staging files older than an hour are removed. Re-uploading the exact original bytes/provenance repairs a missing file without changing the asset ID. Delivery uses immutable private caching, actual MIME types and byte ranges. It shares the local application's access boundary; it is not a substitute for release authentication.

## Contracts, checks and packaging

```bash
uv run phrase-gym export-schema frontend/openapi.json
npm --prefix frontend run generate:types
uv run pytest tests/
uv run pyrefly check --project-excludes=tests --python-interpreter-path=.venv/bin/python
npm --prefix frontend run typecheck
npm --prefix frontend test
npm --prefix frontend run build
npx --prefix frontend playwright install chromium webkit
npm --prefix frontend run test:browser
./build.sh
```

The OpenAPI snapshot and generated TypeScript are committed. Tests compare the snapshot to the API; CI regenerates both and rejects drift. Vite build is not used as a substitute for TypeScript checking, following [Vue's tooling guidance](https://vuejs.org/guide/typescript/overview.html). SQLite transaction configuration follows [SQLAlchemy's SQLite guidance](https://docs.sqlalchemy.org/en/20/dialects/sqlite.html).

`build.sh` replaces the inherited unavailable Docker-script placeholder. It builds the client, prepares ignored package static assets, and builds hatchling distributions. Plain `uv build` before client preparation remains a backend-only package and can use an external `--client-dir`. Existing Python/Invoke/version/docs tooling is retained. Client CI adds types, reducer/browser tests and bundled build checks alongside the existing Python matrix. No push, tag or deployment is implied.

See [P2 evidence](evidence/p2-foundation-2026-10-12.md) for actual checks and remaining acceptance gaps.
