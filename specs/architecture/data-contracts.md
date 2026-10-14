# Data model and integration contracts

Updated 2026-10-14. The [external-agent decision](../decision-external-preparation-agent.md) supersedes source-specific and in-app LLM preparation contracts. The current runtime still exposes its earlier workflow pending simplification. Settings, manual packages, immutable content revisions, library reads, reference exposures and media contracts are implemented in P2; pinned sessions, ordered attempt events, recording transfer and review receipts are implemented in P3. See the [core trainer decision](../decision-core-trainer.md). The committed OpenAPI snapshot and generated client types describe the actual API. Companion to the [architecture](../architecture.md) and [learning engine](learning-engine.md). Implement these incrementally; the prototype needs only fixtures matching the core shapes.

## Durable model

Use opaque UUIDs for internal identities; source identifiers are references, not primary learning identities. Persist timestamps in UTC and render them using the learner's timezone. Store language/locale explicitly; do not infer it from the text or hard-code English.

| Entity | Essential fields and invariant |
|---|---|
| `learning_target` | ID, language, kind (`fixed_phrase`, `word_sense`, `frame`), meaning/use identity, current revision, active/suspended status. |
| `target_revision` | Immutable version of the learning objective, required recall material and acceptable equivalence/slot rules. A different sense, fixed expression or required construction creates another target; compatible editorial changes preserve identity. |
| `realization` | Target revision, complete sentence, purpose/context, ordered chunks with stable IDs, required recall chunk IDs, reference audio asset links. Chunks concatenate exactly to the sentence. Reference audio may be imported or generated. |
| `cue_variant` | Realization revision, visible and required hidden chunk IDs, purpose/partner cue, help layers, modality, optional cue audio, competing-completion notes, publication status and provenance. Preserve declared support/context so selection can favour complementary practice without treating arbitrary cue difficulty as equivalent. |
| `source_item` + links | Source namespace, stable external ID, source revision/hash and target links. Unique import identity; two sources may reference one reviewed target. |
| `lesson` + membership links | Stable lesson ID, learning-language association, title, source references and ordered phrase/realization links. A target may belong to several lessons without duplicating its schedule. Language-scoped queries cannot leak another language's phrases. |
| `phrase_search_document` | Proposed versioned text/intent/topic/synonym index tied to stored phrase and lesson IDs. Exact search mechanism and metadata preparation are open; conceptual results must resolve to existing phrases. |
| `dialogue_revision` | Ordered turns, speaker roles, supporting text, target/realization/cue links and audio. References pin revisions, not mutable text. |
| `session` | Learner/device, pinned learning language, immutable manifest, optional per-attempt capture with actual modality/support, policy/schema versions, status and last accepted event sequence. Navigation to another language never relabels an existing session. |
| `attempt` | Unique event/attempt IDs, target and cue revisions, review group, original-attempt/repair link, actual response modality (recorded speech / silent / explicitly reported unrecorded speech / unknown), exposed chunks and help timing relative to response, media failures, original recall result (`not_yet`, `with_effort`, `recalled`, or no grade), assessment source, eligibility reason, timestamps and optional recording ID. Practice requests and result corrections are distinct events. |
| `review_state` | Unique `(learner_id, target_id)`, due time, last applied review time, versioned library memory state (FSRS difficulty/stability and required state fields under the current recommendation), parameter/policy version, desired retention and concurrency version. Persistent chunk coverage is linked evidence, not interval levels or sibling schedules. |
| `review_receipt` | Unique review group per learner/target; accepted attempt, schedule effect and before/after version. |
| `media_asset` | Reference/cue audio: hash, MIME type, byte size, duration, storage key, language, provenance (imported course/human recording or generated speech, with source/performer or provider details when available) and readiness. Original and normalized versions are linked. Do not infer AI origin from an old field/UI term. Learner recordings use database audio storage below. |
| `recording` | Stable take ID, learner, target/session/attempt and cue/content revision links, captured timestamp, audio bytes (BLOB in SQLite), hash, MIME type, byte size, duration when known, capture/completeness status and persistence acknowledgement. Support/exposure is retained through attempt history. Every take is automatically kept; no saved/pinned opt-in flag. A new take gets a new ID and does not replace earlier audio. No recording is required for a recording-off attempt. |
| `content_job` | Focused reference-audio work and recovery only in the accepted boundary. The earlier LLM job implementation/migration remains historical until simplification. |
| `cue_report`, `learner_note` | Cue-specific quality feedback and target-specific personal notes; neither is a schedule. |
| `app_settings` | Native language collected on first launch and `ui_language_mode` (`native` or `learning`). Bare-URL language restoration is best-effort browser storage, outside this server row. Explanation language derives from native language; effective UI language derives from the mode and current URL's learning language. No independent arbitrary UI locale selector. These two fields are the implemented Settings contract. |

Compatible editorial/context revisions keep the target and schedule; retain recall coverage only for unchanged required chunks, with an explicit mapping for renamed chunk IDs. A source edit must not silently replace the required expression or meaning.

For the accepted external-agent flow, store submitted checked cue variants with the phrase revision, retaining external prompt/model provenance when supplied. Phrase Gym does not own the LLM job. Persist cue presentation in session/attempt history, including a displayed cue on a subsequently abandoned attempt, so selection can vary the hidden parts across sessions. Store zero, one or several candidates as appropriate; only reviewed usable cues enter retrieval practice. These variants do not create additional targets or schedules. The exact history schema and selection policy remain implementation proposals.

The [current review recommendation](../design/answer-card-and-review-policy.md) maps eligible original results to FSRS Again/1, Hard/2 and Good/3. Preserve the state before/after each applied observation for correction and reproducibility. Log separated retries, supported takes and search/explanation exposures even when they do not update the schedule. The first eligible result per review group remains authoritative; later success never overwrites it. Cue selection uses persistent hidden-chunk practice/recall evidence, including support and last presentation, rather than a mandatory coverage gate at every interval. These fields support normal learning; they do not require an experiment or calibration feature.

A singleton learner profile is enough for the personal app. Do not add public account management, organizations or shared courses to the first implementation.

The accepted first-launch flow collects native language when the database is empty and persists it in app settings for explanations. A previously saved setting is reused; an empty phrase collection alone does not require asking again. Proposed content shape: a versioned explanation linked to the realization/target revision and explanation locale, with meaning/translation and usage information. Record opening the explanation and full-model exposure on the existing attempt, preserving whether it happened before or after the response. Keep that content out of the ordinary drill until explicitly requested. The exact explanation schema and preparation/caching policy are still proposals.

### Learning language and interface locale

The [language decision](../decision-languages-and-audio-labels.md) requires a learning-language selector with bookmarkable URLs, separate from the native explanation language. Recommended route prefix: `/learn/{language}` with stable language codes. An explicit route is authoritative over device-local language restoration, including across tabs, reload and Back/Forward. Include the language in lesson/phrase routes and read/session requests; validate that selected content belongs to it. Derive UI locale per tab from `ui_language_mode`; use native language for explanations regardless of that mode. UI preference changes do not modify target identity, recordings or review state.

### Lesson selection and reference lookup

The accepted [navigation/search requirements](../decision-session-navigation-and-search.md) add lesson-title substring filtering, conceptual phrase lookup and full lesson phrase lists. Proposed API reads are `GET /api/v1/lessons?language=en&q=...`, `/lessons/{id}/phrases?language=en` and `/phrases/search?language=en&q=...`; use stable identities and paginate without restricting a lesson list to due items. Session creation carries learning language and explicit lesson scope, All lessons by default or one lesson ID. All lessons uses the spaced-repetition planner within that language. Selected lessons also use due/new eligibility under the user's 2026-10-12 delegation; schedules remain target-owned.

Lookup does not require a scheduled session or attempt. Proposed extension: a practice/exposure context records full phrase/model/explanation exposure from lookup and lesson browsing, with an optional scheduled-session/attempt link. Optional explanation recordings outside a drill attach to this context and target; do not fabricate a review group to satisfy a foreign key. Keep audio retention identical. Actual content exposure may inform later support history, but merely searching, opening a phrase or listing a lesson must not generate a success receipt or advance due dates. Browsing must not take over or discard an active scheduled session. Lookup takes now attach to actual exposure receipts; session takes attach to attempts and optional capture-event receipts. Resume/takeover and pinned-language navigation are implemented.

## Example realization and cues

```json
{
  "target_id": "example-day-target",
  "target_revision": 1,
  "realization_id": "example-day-line",
  "language": "en",
  "kind": "fixed_phrase",
  "text": "How has your day been so far?",
  "purpose": "Ask about the listener's day up to now.",
  "chunks": [
    {"id": "opening", "text": "How has "},
    {"id": "subject", "text": "your day "},
    {"id": "ending", "text": "been so far?"}
  ],
  "required_recall_chunks": ["opening", "subject", "ending"],
  "cues": [
    {"id": "opening-recall", "visible_chunks": ["subject", "ending"]},
    {"id": "subject-recall", "visible_chunks": ["opening", "ending"]}
  ]
}
```

IDs above are readable fixture identifiers, not the production UUID format. These two masks demonstrate rotation; they do not demonstrate recall of the ending. Add a reviewed cue for that chunk, or retain the coverage limitation. Do not mark a cue published solely because it is structurally valid JSON.

## API boundaries

| Endpoint family | Purpose and contract |
|---|---|
| `GET /api/v1/targets`, `/targets/{id}`, `/dialogues/{id}` | Read published content, source summaries and target progress. Paginate collection queries. |
| `POST /api/v1/imports` | Stage a versioned source package; return import ID and validation report. No direct scheduling effects. |
| `POST /api/v1/imports/{id}/publish` | Publish an explicitly reviewed staged revision. Reject missing media or invalid cue references. |
| `POST /api/v1/sessions` | Implemented: idempotent creation with a caller-issued session ID; pinned manifest/groups/versions/media and optional explicit takeover of the current session. |
| `GET /api/v1/sessions/current` | Implemented: retrieve the active/paused session, independent of viewed language. |
| `PATCH /api/v1/sessions/{id}` | Implemented: versioned pause/resume; retains the immutable manifest and single open slot. |
| `POST /api/v1/sessions/{id}/events` | Implemented: ordered event batch, atomic receipts/evidence/schedule effects and authoritative session state. |
| `GET /api/v1/sessions/{id}/events` | Implemented: ordered receipts for journal reconciliation. |
| `GET /api/v1/sessions/readiness` | Implemented: scoped due/new/preparing counts and next UTC due time, with no writes. |
| `POST /api/v1/recordings` | Implemented: stable-ID original-byte take retention in SQLite; validates request, size, declared MIME and revision/context; stores opaque audio bytes without decoding or container inspection. |
| `GET /api/v1/recordings/{id}` | Implemented: original retained take bytes with byte ranges and private caching. |
| `GET /api/v1/sessions/{id}` | Implemented: retrieve persisted lifecycle state and manifest; cursor, queue, sequence and original/support evidence are included; ordered receipts are available from the events read. |
| `POST /api/v1/cue-reports` | Record ambiguity or inappropriate context without grading a memory failure. |
| `POST /api/v1/recordings`, `GET /recordings/{id}/audio` | Automatically persist every take's audio and metadata in the database, with idempotent upload/attachment and private playback. Acknowledge storage only after commit. Failed transfer retains pending audio and does not discard the practice event. Historical browsing/comparison and deletion UI are not first-version requirements. |
| `GET /api/v1/media/{id}` | Private media delivery with byte-range support, MIME type and immutable content identity. |
| `POST /api/v1/preparation`, `GET /api/v1/preparation/{id}` | Existing legacy LLM workflow, superseded and awaiting removal. Actual current signatures remain in OpenAPI; this is not the accepted intake boundary. |

Validate schema versions, ownership, byte limits, state transitions and revision references at the boundary. Render imported/generated content as text or sanitized supported markup; do not execute source HTML or instructions. Import packages contain asset references or uploaded files, not arbitrary server-fetch URLs.

### Event processing

An event envelope includes a stable ID, monotonic sequence, manifest step, event kind, occurred time and structured observations. The server derives attempt/target/cue/group/base-schedule bindings from the pinned step; the client cannot choose authoritative due dates or grades. The client submits observations, never an authoritative due date or algorithm grade.

Within one transaction, validate sequence and the manifest, insert the event, derive the outcome, consume a review group if eligible, compare the schedule version, update the schedule and persist its receipt. If an event ID already exists with the same payload hash, return its original receipt. If it has different content, return conflict. A missing sequence returns the expected sequence; do not silently drop preceding help events.

The first release permits one active session per learner. A takeover closes the previous session for scheduling; late events may be retained as historical practice with an explicit no-schedule-change reason. Changed/withdrawn cues and stale target revisions similarly cannot overwrite the current learning contract.

## Generic prepared-material intake

Phrase Gym provides one input schema and preparation/submission prompt. The user's external agent knows the source format, prepares the selected meaning, chunks/cues, native help, concepts and complete dialogues, and submits optional verified reference audio. Vaults, echo-words and other sources use the same input rules; there is no source-specific sender or parser inside Phrase Gym.

Use stable submission/source identifiers for idempotent retries and target preservation. Keep meanings explicit: two identical surface forms may have distinct targets, and changing a meaning cannot reuse another target's schedule. Compatible edits preserve recordings, unchanged chunk evidence and pinned sessions. An external agent's current note position or selected-sense array index is not a learning identity.

Structural and media-binding validation, atomic content storage and clear receipts/errors remain inside the app. LLM calls/reviews/costs and preparation/delivery retries are external. There is no application-owned human approval workflow. Prepared content quality is the external producer's responsibility, guided by the prompt.

Optional supplied audio must carry exact text/language/role/provenance and uploaded bytes. Preserve supplied originals; reject invalid or supplied-but-untransferred media rather than silently generating replacements. Do not fetch arbitrary URLs. Phrase Gym generates genuinely missing full-phrase/turn audio using its language-specific primary and fallback.

Current generic manual-package/API contracts are implemented; the old special preparation/handoff endpoints still await removal. The accepted prompt/schema delivery and simplified intake path must be implemented against the generated API, without inventing unsupported endpoint names in this document.

## Connected session journal

The live server owns session progress, support, repairs and results. IndexedDB retains acknowledged checkpoints and initiated event/take/start/transition payloads; small lookup exposures use separate localStorage keys. Both retain original IDs/payloads for retry, without offline restoration or local advancement. A receipt is authoritative; audio is removed only after matching ID/hash/size/completeness acknowledgement. Quota failures retain available volatile audio with visible guidance to keep the tab open.

No downloads, app shell/media caches or routine service-worker registration remain. The previous worker URL serves a bounded retirement worker; cleanup deletes only owned caches and obsolete pack/display fallback keys, preserving pending evidence and acknowledged history. It never automatically reloads a live recording. A narrow decoder retains historical event payload identity; new session contracts omit repair deferral. See the [online-only decision](../decision-online-only.md).

## Media and jobs

For model/cue media files, write to a temporary file, validate size/type/decoding, compute its hash and move it atomically before committing a ready reference. Handle an orphaned file or a missing file after a crash through reconciliation. Keep media garbage collection away from active sessions and backup snapshots.

For generated reference audio, retain the exact speech input separately from displayed phrase text, including any Serbian script conversion, its version, engine/voice, language/locale, accent and generation settings. These participate in job/media identity so changed voices or text cannot reuse stale results. Actor/course assets preserve source provenance and original bytes, with any processed playback version linked separately. The [echo-words inspection](../reference/echo-words-audio.md) explains why language/text-only cache keys and sample-rate-based origin inference should not be copied.

For learner takes, persist original audio bytes and metadata together. Validate request, size, declared MIME and context under the [retention decision](../decision-learner-recording-retention.md). Store opaque bytes without decoding or container inspection; only the frontend plays audio. Preserve browser capture completeness, including interrupted fragments, without inferring playback compatibility from contents. Use the same take ID on retry so a lost acknowledgement does not duplicate a recording. Distinct takes remain distinct even if their audio hashes match. Recording storage does not itself advance a review schedule. No automatic expiry or cleanup of retained database recordings is part of the policy.

Focused audio work claims its durable task in a short transaction, then synthesizes outside the lock. Renew the lease for long work. After a crash, expiry permits retry; the canonical input hash deduplicates publication, even if a speech request was repeated. Store bounded error details, expose a retry action and cap provider work. Jobs do not carry database locks across network calls.

## Comparison sequence contract

Recording mode must not be inferred merely from the presence or absence of a recording row. Recording is optional per attempt; record actual capture outcome, model exposure timing and learner self-report separately. There is no recording-mode switch. The UI checkpoint includes the current comparison phase (own playback, model reveal/playback, self-check) so reload or an interrupted take cannot invent a finished response or erase assistance. All audio takes are automatically retained in the app database, including rerecordings, with no manual Save action or automatic expiry. Historical progress-comparison UI is deferred. This storage requirement does not authorize automatic external AI processing.

The [device-language and Settings decision](../decision-device-language-and-settings.md) removes the server language preference and navigation writes. Migration 0002 removes the legacy JSON field.


## Superseded implemented LLM boundary (2026-10-14)

The following legacy runtime remains until simplification; it is not the accepted architecture.

`preparation_job` persists canonical source/settings/prompt identity, base source bindings,
leases/tokens, attempts/retry state, retained raw generation/advisory responses, checkpointed
media IDs, operator corrections and exact review/publication receipts. `source_handoff` binds
an export UUID to the immutable selected-content hash and durable job ID. Receipt/enqueue
and reviewed content/job publication each own a single transaction.

Initial operator human content/listening review is hash-bound to the exact result. The model's
review is advisory; malformed output and uncertain content remain withheld. Zero accepted
cues preserves supported lookup and incomplete retrieval readiness. One reliable cue and
multiple comparable masks are supported. Native-language help is merged without replacing
other reviewed languages. Old target revisions and session manifests remain immutable.
See [P5 runbook](../plan/p5-runbook.md) and [measurement evidence](../plan/evidence/p5-implementation-2026-10-14.md).
