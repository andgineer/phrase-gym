# Architecture and durability

## Product names and responsibilities

Phrase Gym is the product and learning frontend. Phrase Gym Server is the lightweight core and database host. Phrase Gym Coach is the laptop application that prepares learning material and reference speech. Use Coach as its short name within the product. Inbox names the durable submission queue and API domain, not a separate application.

## Phrase Gym Server and Phrase Gym Coach

Phrase Gym Server serves the existing Vue/TypeScript frontend and HTTP API from the same origin. It owns learning sessions, scheduling, library reads, recordings, validation/publication and the inbox API. The core can run on an independently available private host; laptop preparation availability never gates published practice. Phrase Gym Coach runs the preparation and reference-speech workers on the laptop.

**Server invariant:** the server is an extremely lightweight, deterministic storage and practice service. It serves Coach-prepared content and published audio for the browser to play; it never generates missing material on demand or in background jobs. No TTS, LLM, speech recognition, model inference or preparation-agent subprocess runs in the server. Its runtime package contains no model weights, provider SDKs or preparation engines, and its configuration requires no provider credentials. Missing preparation remains queued for Coach. Database scheduling, evidence validation, bounded media validation, publication and job coordination remain server responsibilities; lightweight does not mean trusting client grades or worker output.

One core-owned SQLite database stores all durable state: content, schedules/history, learner recording BLOBs, core settings, personal preferences and revisions, inbox entries/generations, preparation/speech jobs, staged-package metadata and receipts. Reference media and verified inbox attachments use core-owned hash-addressed files with database references. Keep SQLite on the core host's local disk; no shared network SQLite file, second inbox database or replicated preference store. Coach and external agents read and persist through authenticated core APIs, including the inbox domain's material and preferences endpoints. Temporary laptop files are staging/cache only, never the sole acknowledged copy of input or recovery state.

The core stores submissions even while the laptop sleeps. Coach claims jobs when connected, prepares outside transactions and submits staged outputs for validation/publication. Loss of the core blocks new claims and acknowledgements; temporary results may be retained for retry, but an expired or superseded claim cannot publish. Provider connectivity is required only for work that uses that provider. [Online-only practice](decision-online-only.md) refers to the live core, not an always-connected laptop.

Transport/UI adapters depend inward on contracts and focused domain services. Inbox storage, preferences, job leasing, validation, publication, library reads, planning, evidence processing, scheduling and media transfer have separate responsibilities. Each write workflow owns one transaction; helpers receive it. Construct services at the relevant application/process boundary. Sharing a database does not justify a catch-all service or direct worker access to learner data. Code and generated contracts own module layout, signatures and endpoints.

## Implementation language

Rust is the target implementation language for both Server and Coach; retain the Vue/TypeScript client. Coach has a Windows/macOS desktop GUI with the product and lifecycle boundary defined in the [Coach decision](decision-external-preparation-agent.md#desktop-application). Use separate binaries with shared contracts and focused domain modules, keeping Coach dependencies out of the server build. Replace the project's Python implementation and scaffold rather than maintaining a permanent second backend or a Rust wrapper around Python. External agent CLIs and speech tools remain separately installed Coach dependencies; rewriting their internals is outside this migration.

The rationale is a small server runtime and explicit process/resource ownership, with Coach chiefly coordinating APIs, files and subprocesses. Rust supports the required [asynchronous subprocess control](https://docs.rs/tokio/latest/tokio/process/index.html) and [SQLite access](https://docs.rs/rusqlite/latest/rusqlite/). This is an operational choice, not evidence that learning policy or recovery is trivial, or a measured performance claim. Preserve scheduling semantics, immutable evidence, transaction boundaries and existing data during the migration. Measure the server footprint separately from external Coach tools before private release.

## Identity and transaction constraints

Targets represent meanings, not source positions or surface-text similarity. Compatible edits create immutable revisions; changed meanings create targets. Lesson/dialogue appearances share identity and scheduling. Preferred alternatives retain their own history. Dialogue revisions and session manifests pin content/media, prompts, review groups, personal-preference context and policy/schedule versions so later edits cannot rewrite an earlier attempt.

One scheduled session may be open at a time. Start explicitly replaces the known current session; unknown concurrent changes conflict. Session URLs select the displayed session independently of the launcher's open session. Superseded sessions and late reports remain readable history, with schedule changes guarded against stale content/policy/schedule versions and target retirement/suspension/preference changes. Restoration never replays old grades.

An ordered immutable event and any eligible schedule change commit atomically with the receipt. Identical event-ID/payload retries return the same receipt; changed payloads conflict. The server derives bindings from the pinned step and never accepts client-authored due dates or authoritative grades. Bound client time by session/receipt time with five minutes of skew allowance and monotonic adjustment, retaining corrections. Consume each eligible review group once.

Inbox receipt commits durable input and verified attachment references before acknowledging ownership. Recreate atomically suspends the defective revision and records its request. Jobs pin immutable generations and preference revisions; a fenced claim excludes stale results. Package publication uses stable submission identity, checks expected target revisions and commits content plus receipt atomically. A crash after commit returns the existing receipt on retry. Reject changed payloads under the same identity. See the [preparation decision](decision-external-preparation-agent.md) for lifecycle and capability constraints and [personal preferences](decision-personal-preferences.md) for profile updates.

Browser actions stay bound to their initiating card through asynchronous work. The browser journals evidence; it never owns schedule calculations or offline progression. [Learning policy](decision-core-trainer.md) and [recording retention](decision-recording-and-comparison.md) define the evidence boundaries.

## Reference media and speech

Stage, validate and hash uploaded reference bytes at the core, then atomically move them before committing a ready reference. Include provenance/preparation settings in identity; do not infer origin from sample rate. Reconcile missing/orphaned files without discarding originals. Publication checks exact text, language and role; prompt audio must not reveal the answer. Keep active-session, pending-inbox and backup assets out of cleanup.

Coach speech workers claim durable tasks through the core API and synthesize on the laptop outside database locks. Bounded leases/retries and canonical input identity allow crash recovery without duplicate publication or stale-voice cache reuse. Reference playback always uses core-owned published bytes and needs no laptop connection. Learner recordings never enter preparation APIs or provider inputs.

## Private release and recovery

Bind the core behind a private HTTPS boundary restricted to intended devices and worker/API clients. Same-origin mutation checks and trusted host/origin settings are not authentication. Verify actual access rules; broad tailnet membership or untrusted identity headers do not establish authorization. Intake and preferences credentials stay server-side with distinct capabilities. Worker credentials can claim/submit their work and read bounded content candidates, not learner recordings or arbitrary database rows. Preference changes require explicit API authority unavailable to ordinary preparation jobs.

Take consistent backups of the single database, including recording BLOBs, inbox/preferences/jobs and referenced core media, with schema/app/guide versions. Do not copy only the main file of a running WAL database. Restore into isolation and verify content, settings, preferences, queues, schedules, history and playable audio. Invalidate old worker leases on recovery and reconcile pending transfers/receipts before retrying publication. An incompatible migration rolls back through a matching snapshot. Browser journals and laptop caches are not backups. Storage planning includes every take, attachments and staged outputs without introducing automatic recording expiry.
