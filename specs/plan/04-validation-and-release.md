# P4 and P6 — phone reliability, validation and release

Status: **P4 online-only correction and authorized fresh round 5**, updated 2026-10-13. Follow the [online-only decision](../decision-online-only.md), [runbook](p4-runbook.md) and [current handoff](next-session.md). Earlier offline review/evidence remains historical. Actual-device acceptance, P5 integrations and P6 deployment remain separate.

Use the [consolidated plan's acceptance checklist](implementation-plan.md#end-to-end-acceptance-checklist) as release coverage, including language URLs/UI separation, search/navigation, revised recall outcomes, reference audio and every recording's retention. Checks in this file concern implementation reliability, not learner-run learning-effectiveness validation.

## P4.1 — Record, compare and retain every take reliably

Harden the basic recording/comparison adapter already included in P3. Preserve the unified card, optional explicit capture, single microphone/stop control, automatic learner-take → Reference comparison and manual replay/rerecord controls. There is no recording switch. Validate silent self-check without a microphone independently. Ask for the microphone only when the learner chooses recording. Stop tracks on exit; label interrupted/empty clips rather than presenting them as completed speech.

Harden the automatic storage already included in P3: retain every take's audio and metadata in the app database, including retries, with no manual Save step or automatic expiry. Persist pending audio locally, transfer it independently of the attempt event and distinguish pending transfer from acknowledged database storage. Recoverable interrupted takes retain their audio with an incomplete status. Historical progress-comparison UI is deferred; keep the existing immediate learner/reference replay controls.

**Acceptance:** denied/ignored permission does not trap the learner; real recordings play on the supported devices; upload retry creates one recording while a new take creates a separate record; comparison uses the intended reference revision; all takes survive session completion/restart and database restore; quota failure does not claim a recording was saved. No voice is automatically submitted to an external model.

## P4.2 — Temporary server failure

Require the live server for learning/settings. Hide usable practice when a transport/server failure occurs; offer Retry and usable stop/cancel for already active media. Retain every initiated event/take with its original ID/context/bytes. Retry server receipts and reload authoritative state before enabling learning. Do not predict attempts/support/grades or advance locally.

Retire earlier owned worker caches/registrations and obsolete packs without clearing pending journals or reloading active media. No offline packs, cached navigation, installation/update flow or outage platform is in scope. Check representative real-server connection failure, lost response and active media cleanup; preserve established idempotency, capture ownership and conditional audio acknowledgement tests.

## P4.3 — Accessibility and device checks

Repeat critical flows on the actual iPhone in Safari, with speaker and headphones. Check text scaling, focus order, touch targets, readable cue gaps, VoiceOver labels, light/dark contrast, interruptions and visible support state. Test non-Latin and longer-text fixtures where relevant.

Automate authoritative state mapping, persistence and browser state tests, including Chromium and WebKit where supported. Record actual-device audio results separately; an automated desktop WebKit pass cannot stand in for an iPhone recording test. Fix silent failures and misleading saved/graded states before release.

## P6.1 — Choose and prepare the host

Measure the built application and preparation worker on the proposed host, including idle usage, API peak, media-processing peak, disk growth and impact on existing services. Compare the user's Oracle VM and always-on laptop using those observations. Build assets before deployment; do not assume a development build or a local speech model will fit a 1 GB shared machine.

Recommend the smallest private deployment that passes the measurements: one API process, one bounded worker, SQLite/media on local disk and HTTPS reachable only by the intended devices. Confirm host, access scope, backup destination and operational procedure with a concrete deployment proposal. Do not add public ingress, subscriptions or broad network access as incidental implementation work.

**Deliverables:** environment template, start/stop/update commands, health checks, dependency lockfiles, storage limits, logging policy and a host-specific runbook. No secrets or personal takes belong in the repository.

## P6.2 — Backup, restore and upgrade

- Back up SQLite consistently, including every learner-recording BLOB, and include referenced model media, schema version and app version. Verify that all takes, including retries, are retained on restore; historical-comparison UI is not needed to verify audio persistence.
- Restore to an isolated directory/instance and verify content, due dates, attempts, personal notes and playable audio.
- Include every learning language, native-language/UI-mode settings, source mappings, reference-audio provenance and preparation job state in recovery checks. Verify unfinished jobs can recover without duplicating content or replacing an imported actor clip.
- Rehearse a schema upgrade and app rollback. If a migration is not backward compatible, rollback means restoring the matching snapshot, not running old code on new tables.
- Keep worker crash recovery, missing-media reconciliation and retained-write compatibility in the runbook.

**Acceptance:** demonstrated recovery from the backup with matching representative counts and playable media; acknowledged progress survives process restart; a failed update can be reversed without guessing which database/media files belong together.

## P6.3 — Ordinary use and issue correction

The user explicitly rejected having to validate learning effectiveness. Deliver the selected scientifically grounded defaults and let the learner use ordinary sessions. Do not require a personal efficacy pilot, special delayed tests, transfer-assessment sessions, policy comparisons or calibration work. The earlier proposal for such learner work is withdrawn.

The implementation team checks software behavior, content quality and device reliability. User-reported unclear cues and technical faults can be corrected through the normal product controls. Normal review outcomes drive automatic scheduling; different prepared contexts are ordinary learning exercises. Do not claim guaranteed fluency or an optimum from personal usage history.

## Release gate

Release only when:

- the agreed cue/response/help/dialogue flow has been reviewed and works in the implementation;
- native-language setup, separate UI modes, learning-language bookmarks and scoped home/search/lesson routes work together, with all accepted requirements from the master plan covered;
- both fixed phrases and words/frames work through the same target model;
- one schedule per target, support accounting and idempotent event handling pass the critical tests;
- generated material and live source intake pass their separate quality/contract checks for the selected scope;
- real-device audio/recording, connected journal retry and accessibility checks have recorded results;
- private access, backup/restore and rollback are demonstrated on the chosen host;
- remaining limitations are stated concretely and accepted by the user before calling the corresponding feature complete.

The old files and previous visual sketches cannot substitute for any of this evidence. No existing app progress is assumed available to migrate. If migration is later requested, first inventory/export actual state and design that migration separately.
