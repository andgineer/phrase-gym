# P4 and P6 — phone reliability, validation and release

Status: **not started**, 2026-10-11. P4 follows the core trainer and P1 audio probe. P6 follows the selected release scope, including the agreed source handoff. Hosting remains undecided.

## P4.1 — Record, compare and retain every take reliably

Harden the basic recording/comparison adapter already included in P3. Preserve the agreed session switch, visible state, start/stop/cancel, automatic learner-take → model comparison and manual replay/rerecord controls. Validate recording-off silent practice independently. Ask for the microphone only when the learner chooses recording. Stop tracks on exit; label interrupted/empty clips rather than presenting them as completed speech.

Harden the automatic storage already included in P3: retain every take's audio and metadata in the app database, including retries, with no manual Save step or automatic expiry. Persist pending audio locally, transfer it independently of the attempt event and distinguish pending transfer from acknowledged database storage. Recoverable interrupted takes retain their audio with an incomplete status. Historical progress-comparison UI is deferred; keep the existing immediate self/model replay controls.

**Acceptance:** denied/ignored permission does not trap the learner; real recordings play on the supported devices; upload retry creates one recording while a new take creates a separate record; comparison uses the intended model revision; all takes survive session completion/restart and database restore; quota failure does not claim a recording was saved. No voice is automatically submitted to an external model.

## P4.2 — Downloaded sessions and reconnection

- Add the app manifest/service worker and verified prepared-session pack. Cache only selected material; expose download progress and missing assets.
- Journal transitions and attempt results in IndexedDB. A connected session remains usable through a temporary loss of connectivity.
- Reconcile pending events on foreground/resume/retry. Treat server receipts as authoritative and retain unacknowledged events. Do not rely on background sync firing.
- Keep active-session content compatible through an app update. Offer an update after checkpoint/end rather than reloading over a spoken attempt.
- Detect unavailable/evicted local state and explain what can be restored from the server. Browser cache is not the backup strategy.

**Acceptance scenarios:** download completely then disable network; finish and reconnect; retry after server commit but lost response; close/reopen while offline; exhaust storage; change content while a pack is offline; take over on another device; deliver an old outbox later. Each produces one acknowledged attempt history and no duplicate schedule advancement. Preserve recoverable events even if a version cannot be applied.

Only prepared-session completion is promised at this milestone. Do not label arbitrary library access, new imports, generation, or new due-session creation as offline-capable.

## P4.3 — Accessibility and device checks

Repeat critical flows on the actual iPhone in Safari and installed mode, with speaker and headphones. Check text scaling, focus order, touch targets, readable cue gaps, VoiceOver labels, light/dark contrast, interruptions and visible support state. Test non-Latin and longer-text fixtures where relevant.

Automate reducer, persistence and browser state tests, including Chromium and WebKit where supported. Record actual-device audio results separately; an automated desktop WebKit pass cannot stand in for an iPhone recording test. Fix silent failures and misleading saved/graded states before pilot use.

## P6.1 — Choose and prepare the host

Measure the built application and preparation worker on the proposed host, including idle usage, API peak, media-processing peak, disk growth and impact on existing services. Compare the user's Oracle VM and always-on laptop using those observations. Build assets before deployment; do not assume a development build or a local speech model will fit a 1 GB shared machine.

Recommend the smallest private deployment that passes the measurements: one API process, one bounded worker, SQLite/media on local disk and HTTPS reachable only by the intended devices. Confirm host, access scope, backup destination and operational procedure with a concrete deployment proposal. Do not add public ingress, subscriptions or broad network access as incidental implementation work.

**Deliverables:** environment template, start/stop/update commands, health checks, dependency lockfiles, storage limits, logging policy and a host-specific runbook. No secrets or personal takes belong in the repository.

## P6.2 — Backup, restore and upgrade

- Back up SQLite consistently, including every learner-recording BLOB, and include referenced model media, schema version and app version. Verify that all takes, including retries, are retained on restore; historical-comparison UI is not needed to verify audio persistence.
- Restore to an isolated directory/instance and verify content, due dates, attempts, personal notes and playable audio.
- Rehearse a schema upgrade and app rollback. If a migration is not backward compatible, rollback means restoring the matching snapshot, not running old code on new tables.
- Keep worker crash recovery, missing-media reconciliation and download-pack compatibility in the runbook.

**Acceptance:** demonstrated recovery from the backup with matching representative counts and playable media; acknowledged progress survives process restart; a failed update can be reversed without guessing which database/media files belong together.

## P6.3 — Ordinary use and issue correction

The user explicitly rejected having to validate learning effectiveness. Deliver the selected scientifically grounded defaults and let the learner use ordinary sessions. Do not require a personal efficacy pilot, special delayed tests, transfer-assessment sessions, policy comparisons or calibration work. The earlier proposal for such learner work is withdrawn.

The implementation team checks software behavior, content quality and device reliability. User-reported unclear cues and technical faults can be corrected through the normal product controls. Normal review outcomes drive automatic scheduling; different prepared contexts are ordinary learning exercises. Do not claim guaranteed fluency or an optimum from personal usage history.

## Release gate

Release only when:

- the agreed cue/response/help/dialogue flow has been reviewed and works in the implementation;
- both fixed phrases and words/frames work through the same target model;
- one schedule per target, support accounting and idempotent event handling pass the critical tests;
- generated material and live source intake pass their separate quality/contract checks for the selected scope;
- real-device audio/recording, prepared-session recovery and accessibility checks have recorded results;
- private access, backup/restore and rollback are demonstrated on the chosen host;
- remaining limitations are stated concretely and accepted by the user before calling the corresponding feature complete.

The old files and previous visual sketches cannot substitute for any of this evidence. No existing app progress is assumed available to migrate. If migration is later requested, first inventory/export actual state and design that migration separately.
