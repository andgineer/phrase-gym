# Phrase Gym — architecture

Updated 2026-10-14. The [external-agent decision](decision-external-preparation-agent.md) defines the accepted P5 boundary; the previously implemented source/LLM workflow is superseded and still needs removal. The P2 foundation implements the client/API, settings, content imports/library and reference-media storage. P3 now implements the [core trainer](decision-core-trainer.md), including pinned sessions, ordered evidence, retained recording BLOBs and FSRS. Offline delivery is withdrawn. Simplified prepared intake/reference-speech work and deployment remain open. See the [current module boundaries](decision-production-foundation.md). The agreed product baseline is [targeted drilling](decision-targeted-drilling.md) and [optional recording/comparison](decision-recording-and-comparison.md). The name is Phrase Gym. Earlier interface sketches and legacy tooling are not accepted implementations. Start with the [prototyping plan](plan/01-prototyping.md); implementation follows the [roadmap](plan/implementation-plan.md).

## Recommendation

Build a personal, mobile-first web app that requires a live server. Use **Vue 3 + TypeScript** for the interface, **FastAPI + Python** for the API and focused reference-speech work, and **SQLite (including learner-recording audio) + a local directory for model media** for durable storage. Keep it one application with clear internal modules. Prepare learning material before sessions; the core drill must work without an AI request.

These choices are proposed engineering defaults. The browser/audio prototype must demonstrate that they fit the actual phone before they become the production foundation. Hosting and the visual layout remain open.

| Option | What the user gets | Main trade-off |
|---|---|---|
| **Online web app with a small Python backend — implemented** | One app on phone and desktop, durable content and progress, prepared sessions served online. | Real-device audio, interruption and storage behaviour need validation. |
| Native iOS app with the same backend | A platform-specific interface and access to native audio facilities. | A separate client and distribution workflow; consider if the browser fails a required audio capability. |
| Browser-only app | Easy initial distribution and entirely local practice. | Durable cross-device progress, recovery and server-side material preparation become separate problems. |

Vue and FastAPI align with the nearby echo-words codebase without requiring Phrase Gym to share its runtime or database. This is a maintenance judgment, not evidence that another stack could not work.

## System shape

```mermaid
flowchart LR
    S[User's source] --> E[External LLM agent]
    G[Phrase Gym prompt and input rules] --> E
    E --> I[Prepared phrases and optional audio]
    I --> API[Generic intake and validation]
    API --> D[(Content, progress and media)]
    D --> T[Missing reference speech]
    T --> V[Language-specific primary and fallback]
    V --> D
    P[Practice browser] <--> API
    P <--> J[Pending learner events and recordings]
```

A normal session reads published content and writes practice events. Provider outages can delay new material but cannot interrupt an already prepared drill.

## Components and ownership

| Component | Responsibility | Does not own |
|---|---|---|
| Content domain | Target identities, meanings, realizations, immutable revisions, cues, dialogues and provenance. | Learner progress. |
| Session planner | Choose due/new targets and approved cue variants; prepare an immutable session manifest. | Speech grading or generation during a turn. |
| Browser session engine | Present the current step, play media, manage help/recording, pause/resume and journal events. | The authoritative review schedule. |
| Practice service | Validate attempts, deduplicate retries, classify support and commit history. | A claim that a self-report or transcript proves fluent use. |
| Review policy | Maintain one schedule per learner and target from eligible observations. | One schedule per mask, dialogue appearance or source. |
| Media service | Store/serve model audio and every learner take; persist learner audio in the database with capture metadata and reliable acknowledgement. | Content identity, historical progress-comparison UI or automatic pronunciation scores. |
| Intake guide and generic intake | Explain preparation/submission; validate and store prepared packages and optional media. | Source parsing, LLM calls/review/retries or a special sender. |
| Reference speech | Generate/cache missing whole-phrase audio with a per-language primary and matching fallback. | Text generation or linguistic preparation. |

See [learning behaviour](architecture/learning-engine.md) and [data/API contracts](architecture/data-contracts.md) for the boundaries that must remain stable through UI changes.

## Application stack and layout

- **Client:** Vue 3, TypeScript, Vite and Vue Router. Views render authoritative session state; focused composables own route orchestration, audio and journal transfers. IndexedDB retains acknowledged checkpoints and initiated writes/takes for retry. It does not restore an offline trainer. No shell/media cache or new service-worker registration exists; a minimal retirement asset cleans earlier owned caches/registrations.
- **Server:** FastAPI, Pydantic request/response models, SQLAlchemy and Alembic migrations. Generate TypeScript API types from the OpenAPI contract. Use short database transactions; keep network calls and media processing outside them.
- **Runtime:** one API process serving the built client and API from the same origin, plus focused bounded reference-speech work from the same Python package. Audio work may retain durable retry state; text preparation is external. No Redis or distributed job system is needed for the proposed personal workload.
- **Storage:** one local SQLite database with foreign keys, WAL mode and a busy timeout; immutable model media files addressed by hash; all learner-recording audio stored as database BLOBs with links to attempts and capture metadata. Keep every take automatically, including retries; no manual Save flag or automatic expiry. Use a local filesystem for SQLite, not a network-mounted database file.
- **Tooling:** Python environment/lockfile through uv, npm lockfile, Python lint/type checks and pytest, TypeScript checking and Vitest, browser flow tests with Playwright. Pin a tested dependency set during foundation work; this plan does not prescribe unverified latest versions.

```text
specs/                         agreed decisions and current plans
prototypes/drill-flow/         disposable interaction prototype, to be created
prototypes/audio-probe/        real-device capability probe, to be created
frontend/src/
  features/{practice,library,dialogues,content}/
  session/                    pure state machine and local journal
  platform/                   media, IndexedDB, network adapters
src/phrase_gym/
  api/                        HTTP contracts and authentication boundary
  content/                    targets, cues, revisions and imports
  practice/                   sessions, attempts and review policy
  media/                      reference/learner storage and focused speech generation
  intake/                     intended prompt/schema delivery and generic submission
  # Superseded preparation/integrations modules still await removal.
src/phrase_gym/migrations/
fixtures/                     reviewed learning content and edge cases
tests/{unit,integration,browser}/
```

The user-created scaffold in `~/projects/phrase-gym` is the actual starting point: preserve `src/phrase_gym`, hatchling/uv, the rich-click entry point, pytest, Invoke, documentation and CI conventions. Production modules and the client now extend that scaffold; P3 trainer UI, attempts, recordings and review policy are implemented; connected reliability is the P4 scope; the earlier P5 source/LLM/speech implementation is recorded historically; the accepted external-agent boundary still needs runtime simplification. External linguistic preparation and deployment remain separate responsibilities. All development docs belong under `specs/` (plural). The old trainer is outside the new app boundary; a [source-format reference](reference/legacy-tooling.md) and archive preserve the relevant context here. No earlier code or stopped prototype establishes a completed milestone.

## Content and AI

A target represents a particular expression or word sense/construction. Its examples, cue masks and dialogue turns reference that identity. Preserve source references without allowing a source re-import to reset progress. A change in meaning creates a new target; an editorial correction creates a new content revision.

Phrase Gym publishes a source-independent preparation prompt, actual input schema and submission instructions. The user's external agent reads its own source, prepares/checks cues, translations, useful explanations, search metadata and optional dialogues, and submits the result. Provider selection, calls, costs and preparation retries stay outside Phrase Gym.

The app validates structure/bindings and stores compatible immutable revisions without resetting progress. It does not add an in-app linguistic review or operator-approval workflow. Checked variants are stored before practice; zero, one or several reliable cues are allowed. Learner recordings are not sent to a text LLM by the core flow.

Missing reference audio is the separate in-app speech responsibility. Preserve supplied clips and use a configured primary provider per learning language with an appropriate fallback. Cache complete phrase/turn speech before practice; provider outages cannot break prepared drills.

## State, connectivity and recovery

The server owns published content, committed attempts and schedules. The browser owns an unacknowledged local journal until the server confirms receipt. Save a turn's support/exposure history before showing its next state; reopening the page must not turn a model-assisted attempt into a fresh recall success.

The [online-only decision](decision-online-only.md) requires a live server for all learning and settings. Connection failure hides normal interaction and offers Retry plus stop/cancel for active audio. Retry uses original journaled IDs and actual server state before enabling practice; there is no local advancement or completion. Keep this a minimal error/retry path, without health probes or a general availability platform.

Use immutable event IDs and sequence numbers. The server records an attempt and any allowed schedule update in one transaction. Replaying a request returns the original receipt. A stale session may contribute history but must not overwrite a newer schedule. Display pending versus saved progress truthfully. Browser storage is a cache, not a guaranteed backup; no eviction-survival guarantee is made. See [WebKit's storage policy](https://webkit.org/blog/14403/updates-to-storage-policy/).

One active review session per learner is the initial simplification. A second device can resume after synchronization or explicitly take over. Unsent events from an older device remain importable as historical practice; their schedule effects are guarded by the version recorded in their manifest.

## Audio and recordings

Reference audio is imported from actors/courses or generated once and reused. The user rejected Model as an audio-button label; use Reference as the working replacement. Preserve imported speech and its provenance instead of regenerating it merely for consistency. A cue with missing words is its own reviewed audio asset or a carefully prepared sequence of chunks; never play the hidden answer as part of its prompt. Retain audio provenance and target revision. Do not depend on browser speech synthesis as the reference voice.

The user requested inspection of nearby echo-words for multilingual audio. The [source findings and adaptation guidance](reference/echo-words-audio.md) recommend using its per-language approach for missing reference clips: Piper Lessac for US English, Piper Thorsten for German, and Edge Sophie for Serbian with Cyrillic speech input while preserving displayed script. Honor course/accent settings, preserve whole-phrase delivery, and permit only matching-language fallbacks. Prepare/cache clips before practice; record voice, exact speech input and preparation version in media identity. Echo-words' small-server constraints and word-only Commons lookup are not Phrase Gym requirements. These are source-derived implementation recommendations, not a new voice-quality benchmark.

Recording is a session-level optional mode, as agreed in the [recording decision](decision-recording-and-comparison.md). After a take ends, automatically play the learner take, then reveal/play the model, with manual replay/rerecord and interruption controls. Recording-off practice uses Reveal & compare and manual model audio, and may be silent. Capability detection, permission denial, missing hardware and interruptions must leave that non-recording path usable without claiming spoken evidence. Manual Stop recording is the accepted default and the only first-version stopping method; automatic stopping and the app-settings choice of stopping method are deferred beyond the first version. Detect supported recording formats at runtime; store the actual MIME type and validate decoding rather than assuming every browser produces the same codec. Microphone access requires a secure context and user permission. See [getUserMedia](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getUserMedia) and [MediaRecorder format detection](https://developer.mozilla.org/en-US/docs/Web/API/MediaRecorder/isTypeSupported_static).

Prefer foreground practice initially. Background/locked-screen audio, headset controls and eyes-free outcome reporting are prototype questions, not promised browser capabilities. If essential behaviour fails on the user's phone, compare a native shell or native app using the same content and practice contracts before building around an unreliable workaround.

All learner recordings must be retained in the app database from the first version. In this proposed stack, persist audio bytes as BLOBs with MIME type, capture status, timestamps and links to the target/session/attempt/cue/support history. Retain imperfect takes and rerecordings separately. Stage pending audio in IndexedDB until the database acknowledges persistence, with idempotent retries and visible failure recovery. Historical progress-comparison UI is deferred; immediate self/model replay remains core. This accepted storage requirement supersedes the earlier proposal for selected saved takes in media files.

## Access and deployment boundary

The proposed first deployment is private and single-user. Use an HTTPS entry point restricted to the owner's devices, such as the existing tailnet, and bind the app to loopback behind it. Verify the actual access policy at deployment; membership in a broad tailnet alone is not an application authorization rule. Use a scoped integration token for server-to-server intake; keep keys out of the client. Same-origin mutation checks protect browser requests; media uses the same private access boundary. Do not trust identity headers from a directly reachable client.

SQLite/model-media backups and a restore drill precede real progress. Take consistent database backups through the SQLite backup API, including every learner-recording BLOB; include referenced model media and schema/app versions, and restore into a separate directory for verification. Do not back up an active WAL database by copying only its main file. See [WAL behaviour](https://www.sqlite.org/wal.html) and the [backup API](https://www.sqlite.org/backup.html).

Choose the Oracle VM or always-on laptop only after measuring the built app alongside its intended neighbours. No local model inference, speech model or claimed fit within the existing 1 GB VM is assumed. Resource observations, backup location and operational ownership belong to the release plan.

## Technical basis checked on 2026-10-11

- Local echo-words manifests use Vue 3/Vite and FastAPI. Its `src/echo_words/history.py` calls its history **“Restart-ephemeral”**; `/api/words/{entry_id}` can return `410 request expired`. Its `card.py` has structured `Note`, `Meaning` and `Example` data, and its pipeline's `CardStore` is Anki-shaped. An external sender must preserve stable source identity; Phrase Gym does not scrape that endpoint or implement its source adapter. These are inspection findings, not authorization to change echo-words.
- [Vue's TypeScript guide](https://vuejs.org/guide/typescript/overview.html) supports the proposed client tooling and requires separate type checking from Vite's build. [FastAPI](https://fastapi.tiangolo.com/features/) supplies the OpenAPI/Pydantic contract boundary. [SQLAlchemy's SQLite documentation](https://docs.sqlalchemy.org/en/20/dialects/sqlite.html) documents transaction and foreign-key details to verify in foundation tests.
- [Service workers](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API) provide the browser caching mechanism. Earlier offline experiments are superseded; current use is limited to retiring the previous worker.

## Open decisions with planned resolution

| Decision | Proposed starting point | Resolve in |
|---|---|---|
| Interaction layout and help controls | Prototype several concrete flows against the agreed drills; do not inherit an earlier mockup. | P1 |
| Browser versus native client | Online browser client, conditional on required device/audio behaviour. | P1 |
| Review algorithm | Current recommendation: FSRS with 0.90 default retention and explicit recall/support classification, behind a replaceable interface. See the research-based answer-card/review proposal. No learner-run validation. | Design decision / P3 engineering checks |
| First release inputs | Prepared phrases/word senses from the user's external agent through one generic input format. | P1/P5 |
| External-agent guide | Preparation and semantic review happen externally; the app validates/stores submitted material. | P5 |
| Hands-free scope | Foreground first; assess background/headset/locked-screen behaviour after the device probe. Recording retention is already decided: keep all audio in the database. | P1/P4 |
| Host and final access setup | Private HTTPS; measure both named host candidates. | P6 |
