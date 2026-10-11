# Data model and integration contracts

Proposed contracts, 2026-10-11. Companion to the [architecture](../architecture.md) and [learning engine](learning-engine.md). Implement these incrementally; the prototype needs only fixtures matching the core shapes.

## Durable model

Use opaque UUIDs for internal identities; source identifiers are references, not primary learning identities. Persist timestamps in UTC and render them using the learner's timezone. Store language/locale explicitly; do not infer it from the text or hard-code English.

| Entity | Essential fields and invariant |
|---|---|
| `learning_target` | ID, language, kind (`fixed_phrase`, `word_sense`, `frame`), meaning/use identity, current revision, active/suspended status. |
| `target_revision` | Immutable version of the learning objective, required recall material and acceptable equivalence/slot rules. A different sense, fixed expression or required construction creates another target; compatible editorial changes preserve identity. |
| `realization` | Target revision, complete sentence, purpose/context, ordered chunks with stable IDs, required recall chunk IDs, model audio references. Chunks concatenate exactly to the sentence. |
| `cue_variant` | Realization revision, visible chunk IDs, purpose/partner cue, help layers, modality, optional cue audio, competing-completion notes, publication status and provenance. |
| `source_item` + links | Source namespace, stable external ID, source revision/hash and target links. Unique import identity; two sources may reference one reviewed target. |
| `lesson` + membership links | Stable lesson ID, title, source references and ordered phrase/realization links. A target may belong to several lessons without duplicating its schedule. |
| `phrase_search_document` | Proposed versioned text/intent/topic/synonym index tied to stored phrase and lesson IDs. Exact search mechanism and metadata preparation are open; conceptual results must resolve to existing phrases. |
| `dialogue_revision` | Ordered turns, speaker roles, supporting text, target/realization/cue links and audio. References pin revisions, not mutable text. |
| `session` | Learner/device, immutable manifest, requested recording mode and its change events, policy/schema versions, status and last accepted event sequence. |
| `attempt` | Unique event/attempt IDs, target and cue revisions, review group, actual response modality (recorded speech / silent / explicitly reported unrecorded speech / unknown), exposed chunks, help/media failures, self-report, timestamps and optional recording ID. |
| `review_state` | Unique `(learner_id, target_id)`, due time, interval/coverage data, algorithm version, concurrency version. |
| `review_receipt` | Unique review group per learner/target; accepted attempt, schedule effect and before/after version. |
| `media_asset` | Model/cue audio: hash, MIME type, byte size, duration, storage key, provenance and readiness. Original and normalized versions are linked. Learner recordings use database audio storage below. |
| `recording` | Stable take ID, learner, target/session/attempt and cue/content revision links, captured timestamp, audio bytes (BLOB in SQLite), hash, MIME type, byte size, duration when known, capture/completeness status and persistence acknowledgement. Support/exposure is retained through attempt history. Every take is automatically kept; no saved/pinned opt-in flag. A new take gets a new ID and does not replace earlier audio. No recording is required for a recording-off attempt. |
| `content_job` | Kind, canonical input hash, state, lease owner/expiry, retry count, result/error and provider/prompt provenance. |
| `cue_report`, `learner_note` | Cue-specific quality feedback and target-specific personal notes; neither is a schedule. |
| `app_settings` | Persist the native-language selection collected on first launch with an empty database; use it as the explanation language on subsequent starts. Keep it separate from target languages. Exact locale/schema shape is an implementation proposal. |

Compatible editorial/context revisions keep the target and schedule; retain recall coverage only for unchanged required chunks, with an explicit mapping for renamed chunk IDs. A source edit must not silently replace the required expression or meaning.

For the accepted add-time cue-preparation flow, link the LLM preparation job and its checked `cue_variant` results to the phrase's realization revision. Persist cue presentation in session/attempt history, including a displayed cue on a subsequently abandoned attempt, so selection can vary the hidden parts across sessions. Store zero, one or several candidates as appropriate; only reviewed usable cues enter retrieval practice. These variants do not create additional targets or schedules. The exact history schema and selection policy remain implementation proposals.

A singleton learner profile is enough for the personal app. Do not add public account management, organizations or shared courses to the first implementation.

The accepted first-launch flow collects native language when the database is empty and persists it in app settings for explanations. A previously saved setting is reused; an empty phrase collection alone does not require asking again. Proposed content shape: a versioned explanation linked to the realization/target revision and explanation locale, with meaning/translation and usage information. Record opening the explanation and full-model exposure on the existing attempt, preserving whether it happened before or after the response. Keep that content out of the ordinary drill until explicitly requested. The exact explanation schema and preparation/caching policy are still proposals.

### Lesson selection and reference lookup

The accepted [navigation/search requirements](../decision-session-navigation-and-search.md) add lesson-title substring filtering, conceptual phrase lookup and full lesson phrase lists. Proposed API reads are `GET /api/v1/lessons?q=...`, `/lessons/{id}/phrases` and `/phrases/search?q=...`; use stable identities and paginate without restricting a lesson list to due items. Session creation carries an explicit scope, All lessons by default or one lesson ID. All lessons uses the spaced-repetition planner. Selected-lesson eligibility is still open.

Lookup does not require a scheduled session or attempt. Proposed extension: a practice/exposure context records full phrase/model/explanation exposure from lookup and lesson browsing, with an optional scheduled-session/attempt link. Optional explanation recordings outside a drill attach to this context and target; do not fabricate a review group to satisfy a foreign key. Keep audio retention identical. Actual content exposure may inform later support history, but merely searching, opening a phrase or listing a lesson must not generate a success receipt or advance due dates. Browsing must not take over or discard an active scheduled session. The precise context schema and active-session navigation are design work, not implemented contracts.

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
| `POST /api/v1/sessions` | Start/resume a session; return its pinned manifest, review groups, schedule versions and required media list. Creation has an idempotency key. |
| `POST /api/v1/sessions/{id}/events` | Ordered event batch; return per-event receipts, last accepted sequence and authoritative schedule changes. |
| `GET /api/v1/sessions/{id}` | Retrieve the checkpoint/receipts needed to reconcile a resumed client. |
| `POST /api/v1/cue-reports` | Record ambiguity or inappropriate context without grading a memory failure. |
| `POST /api/v1/recordings`, `GET /recordings/{id}/audio` | Automatically persist every take's audio and metadata in the database, with idempotent upload/attachment and private playback. Acknowledge storage only after commit. Failed transfer retains pending audio and does not discard the practice event. Historical browsing/comparison and deletion UI are not first-version requirements. |
| `GET /api/v1/media/{id}` | Private media delivery with byte-range support, MIME type and immutable content identity. |
| `POST /api/v1/preparation-jobs`, `GET /preparation-jobs/{id}` | Durable asynchronous preparation and status. Polling suffices initially; no streaming chat API. |

Validate schema versions, ownership, byte limits, state transitions and revision references at the boundary. Render imported/generated content as text or sanitized supported markup; do not execute source HTML or instructions. Import packages contain asset references or uploaded files, not arbitrary server-fetch URLs.

### Event processing

An event envelope includes `event_id`, `session_id`, monotonic `sequence`, `attempt_id`, target/cue revisions, `review_group_id`, `base_schedule_version`, event kind, occurred time and structured observations. The client submits observations, never an authoritative due date or algorithm grade.

Within one transaction, validate sequence and the manifest, insert the event, derive the outcome, consume a review group if eligible, compare the schedule version, update the schedule and persist its receipt. If an event ID already exists with the same payload hash, return its original receipt. If it has different content, return conflict. A missing sequence returns the expected sequence; do not silently drop preceding help events.

The first release permits one active session per learner. A takeover closes the previous session for scheduling; late events may be retained as historical practice with an explicit no-schedule-change reason. Changed/withdrawn cues and stale target revisions similarly cannot overwrite the current learning contract.

## Source intake

### Vault

Use a local export command operating on explicitly selected notes and assets. Read the vault; write packages and identity mappings in the project/application data directory. Do not require a server to mount the entire vault, watch unrelated notes, or write learning progress into note files.

The existing note format and `phrases.py` can inform a parser, but the old runtime, hint rules and generated page are not the new app. Current numbered phrases can be reordered: do not assume `lesson-id + position` is a permanent identity. Maintain an import manifest with stable IDs and source fingerprints; surface ambiguous reorders/edits for mapping instead of merging by text or resetting progress. Renames and repeated identical text must be covered by fixtures.

### echo-words

The inspected code has structured `Note`, `Meaning`, `Example` objects, but recent `Entry` history is restart-ephemeral. An `entry_id`, selected sense index or rendered HTML response is not a sufficient permanent import key.

Add an explicit adapter when P5 is implemented. It creates a durable outbound envelope at selection time with:

- contract version and a stable export UUID/idempotency key;
- source language, selected expression and selected meaning description;
- complete useful examples and the surface form in each;
- source/provenance metadata, optional model audio files and source revision;
- a stable target mapping after the first successful import.

Persist that envelope before reporting it sent; retry delivery after restarts using the same ID. An acknowledged import may remain **preparing** rather than **ready to practise**. A sense re-analysis requires semantic reconciliation; its new array position cannot silently change an existing target's meaning.

The adapter should be a new output destination, not a Phrase Gym implementation of an Anki-shaped `CardStore` that creates sibling cards. Provide a manual package export/import path first. The later switch from Anki output to Phrase Gym is a user-visible rollout action; do not silently dual-send or remove existing notes. Existing Anki collection migration is separate, unplanned work.

## Prepared-session cache

A download manifest lists the schema version, app compatibility, immutable content/cue revisions and asset hashes/sizes. Mark a pack ready only after required assets are verified. A failed download leaves no misleading offline-ready state.

IndexedDB holds the session checkpoint, immutable attempts pending acknowledgement and every captured recording awaiting database acknowledgement. Cache Storage holds the app shell and downloaded model audio. Quota failure must be visible; never discard an unacknowledged attempt or pending recording just to make cache space. Local copies may be evicted after durable database storage is verified; that does not delete the retained recording from the database. Evict replaceable model assets first. Sync on foreground/resume/retry; do not require background sync support.

Content is immutable within a session. Service-worker updates wait until the session is checkpointed or ended; keep compatible assets for the active pack. A server schema change that cannot read an old event must return a recoverable incompatibility response, not clear the outbox.

## Media and jobs

For model/cue media files, write to a temporary file, validate size/type/decoding, compute its hash and move it atomically before committing a ready reference. Handle an orphaned file or a missing file after a crash through reconciliation. Keep media garbage collection away from active sessions and backup snapshots.

For learner takes, persist audio bytes and recording metadata together in the database. Validate and record format/completeness; preserve available interrupted audio without presenting it as completed speech. Use the same take ID on upload retry so a lost acknowledgement does not duplicate a recording. Distinct takes remain distinct even if their audio hashes match. Recording storage does not itself advance a review schedule. No automatic expiry or cleanup of retained database recordings is part of the policy.

A worker claims a job with a lease in a short transaction, then performs external work outside the lock. Renew the lease for long work. After a crash, expiry permits retry; the canonical input hash deduplicates publication, even if an external provider was called twice. Store bounded error details, expose a retry action and cap provider work. Jobs do not carry database locks across network calls.

## Comparison sequence contract

Recording mode must not be inferred merely from the presence or absence of a recording row. Record the requested mode, actual capture outcome, model exposure timing and learner self-report separately. The UI checkpoint includes the current comparison phase (own playback, model reveal/playback, self-check) so reload or an interrupted take cannot invent a finished response or erase assistance. All audio takes are automatically retained in the app database, including rerecordings, with no manual Save action or automatic expiry. Historical progress-comparison UI is deferred. This storage requirement does not authorize automatic external AI processing.
