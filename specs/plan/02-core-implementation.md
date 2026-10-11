# P2–P3 — production foundation and core trainer

Status: **not started**, 2026-10-11. Dependencies: P1 findings and selected production scope. References: [architecture](../architecture.md), [data contracts](../architecture/data-contracts.md), [learning engine](../architecture/learning-engine.md).

## P2.1 — Application skeleton and contracts

- Extend the user-created `phrase-gym` scaffold, preserving its Python src layout, hatchling/uv packaging, Click entry point, pytest, Invoke and build/CI conventions. Replace placeholder CLI/docs/metadata with real app behaviour at this phase. Add the selected client/server modules and documented run/check/build commands; legacy tooling stays separate.
- Establish the same-origin API/static-client path, explicit runtime configuration and separate application data directory.
- Define Pydantic contracts, generated client types and schema compatibility checks. Keep prototype fixtures usable as contract examples.
- Implement the deterministic client reducer independently of rendering and a clock-injected server domain layer independently of HTTP.

**Acceptance:** clean checkout/environment can run the documented commands; types are checked separately from the client build; invalid requests fail with actionable errors; no provider credentials reach the client. No authentication/hosting assumption is hidden in a development-only convenience.

## P2.2 — Durable content and identities

- Add migrations for targets, immutable revisions, realizations/chunks, cues, source mappings, dialogues and media metadata.
- Implement manual package import: dry-run validation, staged report and explicit publish. Import both fixed phrases and word/frame material.
- Implement duplicate/re-import handling using stable source IDs. Meaning changes are reconciled explicitly; text similarity and array positions cannot silently merge targets.
- Implement collection/detail APIs and the selected prototype's minimal navigation. Default content is target-language; explanatory help remains requested.

**Acceptance:** import twice without duplication; reorder or rename source material without resetting identity; rejected chunk references never enter practice; a source update does not rewrite an earlier attempt's content. At least one fixture in another language catches English-only tokenization assumptions.

## P2.3 — Model media

- Import verified model audio; create immutable hash-based references and private delivery with MIME type and byte-range handling.
- Build gesture-initiated playback controls against real audio. Match cue audio to precisely the material it is allowed to reveal.
- Keep missing/failed media explicit. An item awaiting its required audio is preparing, not ready for its audio-dependent exercise.
- Implement staged writes and reconciliation for a crash between file creation and metadata commit.

**Acceptance:** full model and partial cue audio play on the prototype-supported devices; playing a cue never leaks the hidden answer; stale media cannot be attached to edited text; a missing file offers recovery instead of silently replacing the model with another language/voice.

## P3.1 — End-to-end session

- Implement the session planner and pinned manifest for due/new targets with approved cues, review groups and schedule versions.
- Build introduction, prompt, response, meaningful help, alternative/unclear-cue handling, repair and results from the reviewed P1 flow. Include the session recording switch: recording-on own-take replay → model reveal/playback → self-check, and recording-off silent Reveal & compare with manual model audio.
- Implement pause/resume with a local checkpoint and server session state. Save support/exposure before transitioning.
- Add deterministic cue rotation and delayed retry selection. Do not generate cues in the UI or make an AI request a prerequisite for advancing.

**Acceptance:** complete the seven P1 walkthroughs with durable state. Reload after a model reveal retains assisted status. A cue reported unclear is excluded from subsequent selection pending review. The first complete vertical slice works with fixed fixture content and no connected model provider.

## P3.2 — Review and transaction correctness

- Implement the selected `ReviewPolicy`, initially the proposed conservative policy if P1 confirms it. Version its parameters and outcome mapping.
- Store all attempts; maintain one schedule per target. Track required-chunk coverage without creating sibling schedules.
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

## P3.3 — Specific dialogue rehearsal

- Render the learned exchange and its selected target turns. Sequence partner audio, cue, full learner response and model comparison.
- Preserve readable supporting lines and allowed frame substitutions.
- Add a hard-turn detour that isolates a line, repairs it and restores the dialogue cursor. Use the same target/attempt/schedule services as individual drills.

**Acceptance:** a complete exchange preserves all turns; detours resume correctly; target repetitions do not create extra schedules; recent full-script exposure is recorded as support. No open-ended chat endpoint is added.

## P3.4 — First useful local build

Integrate the library, reviewed content import, due list, complete session, dialogues, contextual explanations and explicit cue reports. Show preparing/ready states and saved/pending outcomes accurately. Handle empty collection, no due items, missing audio and unavailable backend without losing history.

**Exit gate:** the user can learn and revisit both phrase and word/frame targets locally; all critical domain/integration cases pass; the browser suite covers one complete normal session and meaningful failure paths. This is the first useful implementation, not yet the phone/offline release. Basic optional recording and automated comparison belong to this core slice. Durable saving/retention, upload and device resilience arrive in P4; live content sources and AI preparation in P5.

Additional acceptance: the recording-off path never demands microphone permission or claims spoken production. Turning recording off affects subsequent attempts only. An automatic playback failure offers explicit recovery; it does not silently grade the attempt or lose a playable take. Self-check still distinguishes alternative wording from cue ambiguity. A successful rerecording preserves the original attempt and help history.
