# P2–P3 — production foundation and core trainer

**P1 closeout update, 2026-10-12:** use the [next-session handoff](next-session.md) and [unified practice card](../decision-unified-practice-card.md). These supersede older separate recording/practice modes and rating-correction UI below. No saved-rating banner or Undo in the first implementation; correction is deferred. P2 foundation is implemented; real reference-media/content acceptance remains open. P3 has not begun.

Status: **P2 underway; P3 not started**, 2026-10-12. The current implementation and remaining scope are in the [P2 runbook](p2-runbook.md) and [verification evidence](evidence/p2-foundation-2026-10-12.md). Dependencies: P1 findings and selected production scope. References: [architecture](../architecture.md), [data contracts](../architecture/data-contracts.md), [learning engine](../architecture/learning-engine.md).

Implement the coverage in the [consolidated plan](implementation-plan.md); these work packages supply detail, not a smaller product scope. Accepted requirements and recommended defaults retain their distinct status. P3 is the first useful internal build with prepared fixtures/manual packages; the agreed add-time LLM preparation arrives in P5 before claiming the full feature set.

## P2.1 — Application skeleton and contracts

- Extend the user-created `phrase-gym` scaffold, preserving its Python src layout, hatchling/uv packaging, Click entry point, pytest, Invoke and build/CI conventions. Replace placeholder CLI/docs/metadata with real app behaviour at this phase. Add the selected client/server modules and documented run/check/build commands; legacy tooling stays separate.
- Establish the same-origin API/static-client path, explicit runtime configuration and separate application data directory.
- Define Pydantic contracts, generated client types and schema compatibility checks. Keep prototype fixtures usable as contract examples.
- Implement first launch with an empty database: ask for native language and persist it in app settings for phrase explanations. Reuse the setting across restarts; keep target-language metadata separate. The first-launch/settings presentation follows the reviewed prototype.
- Add the accepted learning-language selector and bookmarkable language URLs. Current route recommendation is `/learn/{language}`; explicit URLs override last-used settings. Scope lessons, search and new sessions to that language, preserving it in phrase/lesson routes and browser history. Multiple tabs can show different languages without rewriting one another's route.
- Localize UI controls separately from learning content and explanations. Support native-language UI or UI following the selected learning language; the proposed control is a single settings toggle, not another language dropdown. Explanations always use the saved native language. See the [language/audio decision](../decision-languages-and-audio-labels.md) for proposal status/defaults.
- Implement the deterministic client reducer independently of rendering and a clock-injected server domain layer independently of HTTP.

**Acceptance:** clean checkout/environment can run the documented commands; types are checked separately from the client build; invalid requests fail with actionable errors; no provider credentials reach the client. No authentication/hosting assumption is hidden in a development-only convenience.

Language acceptance: first-launch native language survives restart; direct English/Serbian bookmarks, reload and Back/Forward resolve the URL language; two tabs do not overwrite each other's language view. Native-language UI and learning-language UI both retain native-language explanations. Changing a route does not relabel an existing session or reset progress. On a bare/invalid/empty language route, follow the reviewed restore/selection state without serving another language under the wrong URL.

## P2.2 — Durable content and identities

- Add migrations for targets, immutable revisions, realizations/chunks, cues, source mappings, dialogues and media metadata.
- Implement manual package import: dry-run validation, staged report and explicit publish. Import both fixed phrases and word/frame material.
- Implement duplicate/re-import handling using stable source IDs. Meaning changes are reconciled explicitly; text similarity and array positions cannot silently merge targets.
- Implement collection/detail APIs and the selected prototype's minimal navigation. Default content is target-language; explanatory help remains requested.
- Add lesson identity/membership and the agreed smart phrase lookup: title-substring lesson filtering, phrase-text and intent matching, direct Show & explain, lesson links and complete lesson phrase lists. Choose the search mechanism from P1 query evidence; retain one target/schedule across memberships. Search results must reference stored phrases.

**Acceptance:** import twice without duplication; reorder or rename source material without resetting identity; rejected chunk references never enter practice; a source update does not rewrite an earlier attempt's content. At least one fixture in another language catches English-only tokenization assumptions.

Navigation acceptance: lesson title `rap` filtering and concept-only `greetings` lookup work against stored content within the learning language. Phrase search is independent of the lesson filter. Search → Show & explain → full lesson list → Start lesson drill retains a valid route back and does not grade browsing/exposure. A lesson list contains non-due phrases even when the selected drill policy is due-only. No-match and empty-collection states retain useful navigation.

## P2.3 — Reference media

- Import verified reference audio, including professional-actor/course recordings; preserve provenance and supplied audio rather than replace it with synthesis. Generated audio remains supported. Use Reference / My recording as the working UI labels, localized with the interface preference; create immutable hash-based references and private delivery with MIME type and byte-range handling.
- Build gesture-initiated playback controls against real audio. Match cue audio to precisely the material it is allowed to reveal.
- Keep missing/failed media explicit. An item awaiting its required audio is preparing, not ready for its audio-dependent exercise.
- Implement staged writes and reconciliation for a crash between file creation and metadata commit.

**Acceptance:** full reference and partial cue audio play on the prototype-supported devices; playing a cue never leaks the hidden answer; stale media cannot be attached to edited text; a missing file offers recovery instead of silently substituting another language/voice. Imported actor recordings retain source and original bytes; Reference / My recording labels do not imply synthesis. A headword-only source clip cannot satisfy a whole-phrase reference requirement.

## P3.1 — End-to-end session

- Implement the session planner and pinned manifest for due/new targets with approved cues, review groups and schedule versions.
- Scope Start session to All lessons under spaced repetition by default, or the selected lesson. Use the same scoped entry from Start lesson drill. Resolve due-only versus all-phrase eligibility for a selected lesson before implementing that branch; do not create per-lesson schedules.
- Pin each session's learning language. Preserve an existing session and all recordings when navigating to another language; a viewed language route alone neither starts nor takes over a session. Implement the reviewed pause/resume behaviour without bypassing the single-active-session rule.
- Build one Start session entry, prompt, response, meaningful help, an on-request full-phrase/explanation card in the learner's language, alternative/unclear-cue handling, repair and results from the reviewed P1 flow. No separate home learning/rehearsal modes or compulsory introduction based only on a new database item. Include the session recording switch: recording-on own-take replay → model reveal/playback → self-check, and recording-off silent Reveal & compare with manual model audio.
- Implement pause/resume with a local checkpoint and server session state. Save support/exposure before transitioning.
- Automatically persist every learner take's audio and metadata in the database, including rerecordings. Link each take to its timestamp, target/session/attempt, cue revision and support history. Keep pending audio locally until the database acknowledges it, retry idempotently and never overwrite earlier takes. No manual Save control, automatic expiry or historical progress-comparison UI is required.
- Support recording directly on Show & explain with the complete phrase visible during capture and learner/model comparison. Reuse recording controls, the session switch and storage; persist full-model exposure and classify these as supported practice. Returning to the drill preserves that support and any earlier attempt's separate outcome.
- Keep Show & explain usable when entered through reference lookup outside a scheduled drill. Retain exposure and any optional recordings without manufacturing a review grade/session. Preserve an active drill when browsing its lesson; explicit lesson-start behavior during an existing session must follow the reviewed navigation, not silently abandon work.
- Implement the current proposed result controls: Not yet / With effort / Recalled for the original attempt, separate Practice again and correction through Undo. Other answer / problem is removed by the accepted P1 feedback. Preserve the first result before rerecording. Model-supported practice uses Continue without inventing an independent-recall grade.
- Select clear cues with comparable support that exercise under-practised or difficult hidden chunks, using persistent history rather than cosmetic rotation or a per-interval coverage gate. Include suitable prepared contexts/dialogue turns of the same target in ordinary practice. Do not generate cues in the UI or require a live AI request to advance.
- Implement a separate bounded repair queue: corrective feedback, optional supported practice, then a retrieval retry after intervening time/material. Current defaults are roughly 5–10 minutes and two other targets when feasible, at most two extra retrieval attempts, otherwise continue in a later session. Never show a waiting timer or endlessly loop a failed item; persistent difficulty returns to meaning/cue repair.

**Acceptance:** complete the seven P1 walkthroughs with durable state. Reload after a model reveal retains assisted status. A cue reported unclear is excluded from subsequent selection pending review. The first complete vertical slice works with fixed fixture content and no connected model provider.

## P3.2 — Review and transaction correctness

- Implement the selected `ReviewPolicy`; the latest recommendation is [FSRS with 0.90 default retention and explicit recall grading](../design/answer-card-and-review-policy.md). Version parameters and outcome mapping. Automatic target-state updates use ordinary ratings; no experiment, calibration task or effectiveness-validation feature is required from the learner.
- Store all attempts; maintain one schedule per target. Track required-chunk coverage without creating sibling schedules.
- For the current recommended adapter, use population parameters, Again/1 for Not yet, Hard/2 for correct recall With effort, and Good/3 for Recalled. Disable library short learning/relearning steps to avoid duplicating the app's repair queue. Apply one long-term update per review group; use real elapsed time and library-computed lapse adjustments, without the superseded fixed ladder/reset/30-day cap.
- Permit deliberate silent recall to supply memory ratings with modality retained. Keep reading, model-visible practice, unresolved cue ambiguity and technical failure out of independent-recall grades. Store all exposures and repair events without overwriting the original result.
- Atomically persist event receipt and allowed schedule update. Add idempotency/payload conflict handling, ordered event batches and schedule compare-and-swap.
- Implement one active session, explicit takeover and late-event handling. Establish clock-skew rules and use a controlled clock in tests.

**Acceptance scenarios:**

1. Double tap, request retry after lost acknowledgement and worker/API restart do not advance a schedule twice.
2. The same target in two masks and a dialogue produces one review state.
3. Model exposure, immediate repair and visible target-word use do not count as independent recall or repeatedly lengthen the interval.
4. A later successful repair does not replace the first due-review difficulty; incomplete coverage stays visible.
5. Stale session/content versions cannot overwrite a newer schedule; rejected events remain recoverable where appropriate.
6. An unclear cue, technical failure, valid alternative or skipped item is not graded as forgetting.
7. UTC due times display correctly across timezone/DST changes; invalid client times cannot push a target arbitrarily into the future.
8. With effort is a correct retrieval, not partial recall. Extra practice after a correct result cannot become a failure, and Undo corrects the original event rather than adding a second review.
9. The app repair queue and library steps cannot both schedule the same repair. Two intervening fast cards alone do not satisfy the time-spacing rule; short sessions defer without waiting.
10. Silent retrieval and model-visible practice have different eligibility, and cue selection targets missing coverage without blocking every interval on complete coverage.

## P3.3 — Specific dialogue rehearsal

- Render the learned exchange and its selected target turns. Sequence partner audio, cue, full learner response and model comparison.
- Preserve readable supporting lines and allowed frame substitutions.
- Add a hard-turn detour that isolates a line, repairs it and restores the dialogue cursor. Use the same target/attempt/schedule services as individual drills.

**Acceptance:** a complete exchange preserves all turns; detours resume correctly; target repetitions do not create extra schedules; recent full-script exposure is recorded as support. No open-ended chat endpoint is added.

## P3.4 — First useful local build

Integrate the library, reviewed content import, due list, complete session, dialogues, contextual explanations and explicit cue reports. Show preparing/ready states and saved/pending outcomes accurately. Handle empty collection, no due items, missing audio and unavailable backend without losing history.

**Exit gate:** the user can learn and revisit both phrase and word/frame targets locally; all critical domain/integration cases pass; the browser suite covers one complete normal session and meaningful failure paths. This is the first useful implementation, not yet the phone/offline release. Basic optional recording, automated comparison and durable database storage of every take belong to this core slice. P4 hardens device/offline resilience and transfer recovery; live content sources and AI preparation arrive in P5. Historical recording-comparison UI is deferred.

Additional acceptance: the recording-off path never demands microphone permission or claims spoken production. Turning recording off affects subsequent attempts only. An automatic playback failure offers explicit recovery; it does not silently grade the attempt or lose a playable take. Self-check still distinguishes alternative wording from cue ambiguity. A successful rerecording preserves the original attempt and help history.

Recording-storage acceptance: initial takes and retries remain as separate database records with their audio after session completion and application restart; resending the same take creates no duplicate. Recoverable interrupted audio is marked incomplete. Storage failure never claims a successful save, and no test relies on a manual Save action or a historical comparison screen.
