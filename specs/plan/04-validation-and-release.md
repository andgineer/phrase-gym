# P4 and P6 — phone reliability, validation and release

Status: **not started**, 2026-10-11. P4 follows the core trainer and P1 audio probe. P6 follows the selected release scope, including the agreed source handoff. Hosting remains undecided.

## P4.1 — Record, compare and keep selected takes

Harden the basic recording/comparison adapter already included in P3. Preserve the agreed session switch, visible state, start/stop/cancel, automatic learner-take → model comparison and manual replay/rerecord controls. Validate recording-off silent practice independently. Ask for the microphone only when the learner chooses recording. Stop tracks on exit; label interrupted/empty clips rather than presenting them as completed speech.

Persist a take locally before offering it as saved; upload it independently of the attempt event and show pending/saved state. Define a proposed retention policy in the prototype review: keep explicitly saved/pinned takes and an opted-in baseline recording; discard unsaved temporary takes after comparison/session completion. Offer deletion without deleting the learning target or its textual practice history.

**Acceptance:** denied/ignored permission does not trap the learner; real recordings play on the supported devices; upload retry creates one recording; comparison uses the intended model revision; quota failure does not claim a recording was saved. No voice is automatically submitted to an external model.

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

- Back up SQLite consistently and include all referenced media, schema version and app version. Keep an explicit policy for saved takes and later deletions in backups.
- Restore to an isolated directory/instance and verify content, due dates, attempts, personal notes and playable audio.
- Rehearse a schema upgrade and app rollback. If a migration is not backward compatible, rollback means restoring the matching snapshot, not running old code on new tables.
- Keep worker crash recovery, missing-media reconciliation and download-pack compatibility in the runbook.

**Acceptance:** demonstrated recovery from the backup with matching representative counts and playable media; acknowledged progress survives process restart; a failed update can be reversed without guessing which database/media files belong together.

## P6.3 — Personal pilot

Run a small reviewed set through multiple real sessions over several days. Track cue clarity, successful production with the exact support shown, requests for extra help, delayed recall, changed-mask performance, dialogue use, retries and technical interruptions. Separate preparation quality from learner performance.

Ask the learner about friction, misleading prompts and whether the drilled expressions become easier to retrieve in the intended situations. Optional transfer checks must allow valid alternative wording and cannot define failure merely by absence of one selected sentence. This is a personal product pilot, not a controlled efficacy study.

Refine cue rotation, coverage rules, interval defaults and session size from observations. Preserve policy versions so results remain interpretable. Do not advertise automatic fluency or an optimal schedule from a handful of sessions.

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
