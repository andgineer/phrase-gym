# Architecture and durability

Use the existing Vue/TypeScript client and Python API as one personal application, serving the built client and API from the same origin. SQLite stores content, progress and learner audio; local hash-addressed files store reference media. Keep SQLite on local disk. The application runs on the learner's laptop together with the [preparation inbox and agent](decision-external-preparation-agent.md).

Transport/UI adapters depend inward on contracts and focused domain services. Validation, publication, library reads, planning, evidence processing, scheduling and media transfer have separate responsibilities. Each write workflow owns one transaction; helpers receive it. Construct services at the application boundary. Code and generated contracts own module layout, signatures and endpoints.

## Identity and transaction constraints

Targets represent meanings, not source positions or surface-text similarity. Compatible edits create immutable revisions; changed meanings create targets. Lesson/dialogue appearances share identity and scheduling. Dialogue revisions and session manifests pin their content/media, prompts, review groups and policy/schedule versions so later edits cannot rewrite an earlier attempt.

One scheduled session may be open at a time. Start explicitly replaces the known current session; unknown concurrent changes conflict. Session URLs select the displayed session independently of the launcher's open session. Superseded sessions and late reports remain readable history, with schedule changes guarded against stale content/policy/schedule versions.

An ordered immutable event and any eligible schedule change commit atomically with the receipt. Identical event-ID/payload retries return the same receipt; changed payloads conflict. The server derives bindings from the pinned step and never accepts client-authored due dates or authoritative grades. Bound client time by session/receipt time with five minutes of skew allowance and monotonic adjustment, retaining corrections. Consume each eligible review group once.

Browser actions stay bound to their initiating card through asynchronous work. The browser journals evidence; it never owns schedule calculations or [offline progression](decision-online-only.md). [Learning policy](decision-core-trainer.md) and [recording retention](decision-recording-and-comparison.md) define the evidence boundaries.

## Reference media and speech

Stage, validate and hash reference bytes, then atomically move them before committing a ready reference. Include provenance/preparation settings in identity; do not infer origin from sample rate. Reconcile missing/orphaned files without discarding originals. Publication checks exact text, language and role; prompt audio must not reveal the answer. Keep active-session/backup assets out of cleanup.

Speech work claims a durable task in a short transaction and synthesizes outside locks. Bounded leases/retries and canonical input identity allow crash recovery without duplicate publication or stale-voice cache reuse. Text preparation belongs to the [preparation agent](decision-external-preparation-agent.md), which Phrase Gym starts as a separate process; inbox storage, agent runs and intake publication are separate responsibilities.

## Private release and recovery

Bind the app to loopback behind a private HTTPS boundary restricted to intended devices. Same-origin mutation checks and trusted host/origin settings are not authentication. Verify actual access rules; broad tailnet membership or untrusted identity headers do not establish authorization. Intake credentials stay server-side.

Take consistent SQLite backups including recording BLOBs, referenced media and schema/app versions. Do not copy only the main file of a running WAL database. Restore into isolation and verify content, settings, schedules, history and playable audio. An incompatible migration rolls back through a matching snapshot. Browser journals are not backups.
