# Phrase Gym — consolidated implementation plan

**Latest accepted interaction, 2026-10-12:** follow the [unified practice-card decision](../decision-unified-practice-card.md). One card retains recording, replay, reference and colored/icon self-check controls; answer reveal replaces gaps, explanation expands inline beside Show answer. Remove the recording-mode switch, Practice again and supported-only Continue. The microphone/stop icon reflects the active audio action. This supersedes conflicting earlier flow details below.

Updated **2026-10-13** with the required LLM phrase-preparation prompt-design work. Originally consolidated on 2026-10-11 at the user's request to put everything discussed into the implementation plan. This is the main execution handoff; the linked phase files contain detailed work and checks. Start with [START-HERE](../START-HERE.md).

**Status, 2026-10-13:** P1 prototype/design work is closed at the user's request; prototype 0.1.6 captures the final interaction. P2 engineering work and its review loop are closed at the user's request: the production foundation, library/import contracts and reference-media path are implemented; P3 core engineering implementation and its final gate are complete: card, events, recordings, FSRS, repairs and prepared dialogues are integrated; see the [core decision](../decision-core-trainer.md) and [P3 evidence](evidence/p3-core-2026-10-12.md). See the [P2 closeout](evidence/p2-closeout-2026-10-12.md) and [open problems](../open-problems.md), including broader device acceptance after the successful production Safari/M1 recording report on 2026-10-13. See [the runbook](p2-runbook.md) and [foundation evidence](evidence/p2-foundation-2026-10-12.md). Read [the next-session handoff](next-session.md) and [closeout evidence](evidence/p1-closeout-2026-10-12.md). P4 reliability implementation and verification are recorded in [P4 evidence](evidence/p4-core-2026-10-13.md) and the [offline-session decision](../decision-offline-sessions.md); its authorized fresh review is tracked separately. Device/audio/content gaps are carried forward explicitly, not claimed passed. The unified-card decision supersedes older interaction requirements below, including separate practice/recording modes and Undo.

**Status convention:** **Agreed** identifies explicit user requirements. **Recommended** identifies the concrete design/engineering choice to carry into the next prototype, without claiming the user already selected its exact controls or values. **Open** identifies remaining product choices. No learning-effectiveness experiment, calibration task, special assessment session or algorithm tuning is required from the learner.

## Product and agreed behaviour

### 1. Learning targets and session entry

**Agreed:** train selected words in useful phrases, fixed expressions/constructions and bounded prepared dialogues toward ready use. The response is the whole phrase. Fixed expressions have explicit acceptable equivalents; word/frame targets define the material to retrieve and what surrounding wording may vary.

There is one **Start session** action. No separate Learn new material/Rehearse modes and no compulsory introduction merely because a phrase is new to the database. Expect prior source exposure, but always support forgetting or not understanding through Show & explain. Different cues, examples, lesson memberships and dialogue appearances share **one target and one review schedule**. Different meanings are different targets.

Open-ended AI conversation, free role-play and a general speaking partner are excluded, including as later directions. See the [targeted-drilling decision](../decision-targeted-drilling.md).

### 2. Native language, learning language and UI

**Agreed:** ask for native language on first launch with an empty database, save it in app settings and reuse it for explanations. Russian is the current sketch's explanation language, not a universal default.

Provide a learning-language selector: the learner studies several languages concurrently. Store the selected language in the URL so bookmarks open a specific language. From the very first version, UI controls can use the native language or the selected learning language. Do not add an independent dropdown of arbitrary UI languages. Phrase/situation/partner-line content stays in the learning language; explanations stay native even when buttons use the learning language.

**Recommended:** header language selector; `/learn/en`, `/learn/sr` and equivalent stable-code routes; browser titles such as Phrase Gym: English. A settings toggle, **Use learning language for the interface**, expresses the two modes, initially off. Exact route syntax, control placement and preference default remain proposals.

An explicit URL takes precedence over device-local language restoration, including in separate tabs. A bare URL restores a valid best-effort browser value or shows language choice; server Settings stores only native language and UI mode. Scope lessons, lookup and sessions to that language; All lessons means all lessons within it. Preserve the language in phrase/lesson links, reload and Back/Forward. Switching language preserves progress/recordings; pause an active session and offer Resume on return without relabelling its content. Finish/stop capture before leaving. Merely viewing another tab does not take over the active session. See the [language decision](../decision-languages-and-audio-labels.md).

### 3. Home, lookup and lesson navigation

**Agreed home controls:**

- **Lesson to drill:** searchable dropdown, default All lessons under spaced repetition. Title substring filtering: `rap` finds Building rapport and other matching lesson titles. Selecting an option scopes Start session but does not start it.
- **Find a phrase:** a separate search for stored phrase text and concepts absent from the wording, such as `greetings`. Selecting a result opens Show & explain for reference, not a completed review.
- **Start session:** starts the selected scope through the same drill flow.

Show & explain links to the phrase's lesson. That lesson view lists **every phrase**, including ones not due, and includes **Start lesson drill**. Recommended: open each listed phrase in Show & explain; preserve the originating query/list/session on return. Phrase lookup searches across lessons of the learning language independently of the lesson dropdown.

Lookup, opening an explanation and browsing displayed phrases record exposure where appropriate; they never fabricate a recall result or move a due date. Search results must reference existing stored phrases. Recommend prepared intent/topic/synonym metadata plus text search; embeddings are an implementation option, not a requirement. Search in the native language is still an optional unselected extension.

**Selected under user delegation, 2026-10-12:** both All lessons and a selected lesson use due/new phrases. No Practice all control is added. See the [session-planning decision](../decision-p3-session-planning.md). See the [navigation decision](../decision-session-navigation-and-search.md).

The [manual-feedback decision](../decision-p1-manual-feedback.md) supersedes earlier hint/problem-control proposals: label full-phrase help **Show answer**, and remove **Other answer / problem** entirely.

### 4. Compact drill and recording flow

**Agreed:** show a compact situation/partner line and enough meaningful phrase-specific chunks to identify the intended expression. Necessary disambiguation is present initially. Avoid random gaps, first-letter puzzles and broad situations that demand one arbitrary exact sentence. Ambiguous cues and valid alternatives are not automatic failures.

Recording defaults on and remains optional per session, with the switch available during practice. Resume preserves the session’s explicit choice. With recording on:

1. Show the cue and an **icon-only microphone button**, with an accessible Start recording name.
2. Start only on the learner's action. During capture show a stop icon and **Stop recording**.
3. Manual stop ends capture and automatically plays the learner take, keeping the cue visible.
4. After that playback, reveal the complete phrase and play the reference audio.
5. Offer self-check, replay and extra practice. Playback finishing does not itself grade or advance.

Keep the accepted compact presentation. Do not restore “Say the whole phrase aloud.” or “Ready when you are.” Manual stop is the only V1 stopping method. An automatic-stop option and its app setting are deferred.

With recording off, use **Reveal & compare**, permit silent recall and play reference audio only when requested. No microphone permission is required. Do not report silent work as demonstrated spoken fluency. Denied permission, interrupted/empty capture and playback failures offer recovery without a memory-failure grade.

**Wording correction:** Model is rejected as an audio-button label. Use **Reference** beside **My recording** as the current recommended copy, localizing both with the UI. Reference can be an actor/course recording or generated speech. See the [recording decision](../decision-recording-and-comparison.md) and [screen design](../design/interaction-design.md).

### 5. Show & explain and support history

**Agreed:** make prepared help available before any attempt. The phrase-preparation prompt must supply a natural native-language translation/meaning, adding concise usage/nuance or chunk/construction explanation only when useful. A straightforward phrase may need translation alone, with no filler usage section. Show & explain appears only when useful content exists in the selected native language; missing content hides the control rather than returning a not-prepared message. Show answer and reference playback remain separate available actions.

Allow recording directly on this card. The full phrase stays visible throughout capture, learner playback and reference playback. Respect the same recording switch, manual controls and storage policy. These are supported practice takes, including when the card came from lookup outside a session.

Opening help alone is neither failure nor recall success. Preserve when help/reference material was shown relative to the original response; returning to a hidden cue cannot erase that exposure. Help after a response does not retroactively invalidate the earlier response. Recommended return flow: optional supported speaking followed by another retrieval opportunity after intervening material/time. Repeated difficulty returns to meaning or cue repair rather than an endless repeat loop.

### 6. Keep every learner recording

**Agreed from V1:** automatically store the audio bytes and metadata of **every take in the app database**, including imperfect attempts, rerecordings, explanation-card practice and recoverable interrupted clips. New takes never overwrite earlier ones. No Save button, opt-in selection or automatic expiry.

Store take identity, target, session/practice context, attempt, timestamp, content/cue revision, support/exposure, format and capture/completeness status. No-audio capture failures remain events, not fabricated recordings. Keep pending audio locally until database storage is acknowledged; retries must not duplicate it. Show storage failures honestly and preserve recoverable data.

Immediate learner/reference comparison is core. A historical recording timeline, baseline picker or progress-comparison UI is explicitly deferred. Recording retention does not authorize external AI processing of learner audio.

### 7. Prepared cues and dialogues

**Implementation requirement clarified, 2026-10-13:** design and measure the LLM preparation prompt, including whether the actual visible cue identifies the learned phrase against plausible competing responses. Produce reviewed variants that exercise different meaningful parts, not just one nonempty hint or cosmetic prompt changes. Prepare related target meaning, concise native explanations, acceptable equivalents and search metadata in the same intake workflow where appropriate. The [focused preparation plan](phrase-preparation.md) defines prompt/review deliverables, quality measurements and integration acceptance. The current local 44-phrase pack has one manual cue per phrase and does not fulfil alternate-cue preparation.

**Agreed:** when adding a phrase to the database, have the LLM prepare different hidden-word/meaningful-chunk variants where useful alternatives exist. Check and store them for later sessions. Every variant requests the whole phrase and shares its target/history/schedule. A single reliable cue is enough; if no reliable cue exists, retain supported practice and flag preparation work instead of manufacturing an ambiguous test. No runtime random masking or live LLM dependency during a drill.

Prepared dialogues identify learner target turns and supporting lines. Show/play the partner line, cue the learner's whole response, compare, and continue. Supporting lines can remain readable. Isolate a difficult target turn, practise it, and restore its position in the exchange without adding another schedule.

**Recommended learning refinements:** choose a clear, comparably supported cue covering under-practised or difficult chunks; track cumulative coverage without requiring every chunk at every interval. For a word-retrieval task the target form must be hidden. Use suitable different prepared situations/dialogue turns during ordinary reviews, retaining meaning, allowed equivalents and identifying context. These are practice variations, not tests or open-ended conversation.

## Research-based answer card and scheduling recommendation

The user requested the best scientifically grounded practical approach and explicitly rejected having to validate it personally. Carry the following concrete defaults into the prototype; they remain recommendations, not claims that the user selected every label/parameter. The [research review](../research-answer-card-and-scheduling.md), [answer-card specification](../design/answer-card-and-review-policy.md) and [learning engine](../architecture/learning-engine.md) give the rationale and full rules.

### Grade the original attempt

| Result button | What it means | Scheduler rating |
|---|---|---|
| **Not yet** | Could not retrieve required material, or required wording was wrong/missing beyond allowed equivalents. Recognizing it after reveal does not count. | Again / 1 |
| **With effort** | Correct recall using the planned cue, with substantial retrieval difficulty. It never means forgotten or partly correct. | Hard / 2 |
| **Recalled** | Correct recall without substantial difficulty; ordinary hesitation is fine. | Good / 3 |

Keep **Practice again** separate: a desire to improve pronunciation/flow is not a failure. Preserve/rate the original attempt before rerecording; later success cannot replace it. The user removed **Other answer / problem** entirely on 2026-10-12; do not replace it with a skip/report submenu. An allowed equivalent can still be correct; technical failure does not automatically create a memory-failure grade.

Recommend one result tap saves and advances, with Undo correcting the original event. If extra practice was already requested, retain the result before entering practice. Keep criteria in a collapsed What counts? area. No separate Easy button, predicted interval labels, second confidence question, numerical fluency score or cue-to-microphone speed grading.

Help before any attempt uses a supported-practice state with Continue / Practice again, not an independent-recall success. The separate I couldn’t recall it shortcut is removed by user request. Show answer stays inside the drill; Show & explain sits outside it as visually distinct supplementary learning materials. Opening either must not fabricate a failure. Deliberate silent recall before reveal can use the memory ratings with its modality retained; reading, displayed-word use and immediate imitation cannot. Search/explanation exposure is logged without a fabricated grade.

### Schedule and retry automatically

- Use a pinned, versioned **FSRS-based ReviewPolicy with 0.90 desired retention**, population parameters and ordinary automatic target-state updates. The value is a scheduling estimate for the cued task, not a fluency score. The learner neither tunes it nor runs a calibration exercise.
- Use one state/schedule per target and actual elapsed time. Let the library compute interval and lapse changes. Do not restore the fixed 1/3/7/14/30-day ladder, universal one-day lapse reset or permanent 30-day ceiling.
- A new database item can receive an ordinary first retrieval attempt. Import/study alone creates no successful grade. After study without a valid grade, recommend a next-day learning check as a fallback.
- After Not yet, give corrective reference/meaning feedback and optional supported practice. Queue a retrieval retry after **roughly 5–10 minutes and at least two other targets** when feasible; otherwise return in a later session. Do not make the learner wait on a timer.
- Aim for one successful separated retry; cap at **two extra retrieval attempts per target per session**. Persistent difficulty calls for meaning/cue repair and continuation later. Immediate visible-phrase recordings remain optional practice.
- The first eligible finalized result in a review group updates the long-term schedule once. Log all repairs/exposures without repeated promotions or overwriting the original difficulty. Disable library short learning/relearning steps in this adapter so a second queue does not duplicate the app's repair queue.
- Default sessions draw due material across lessons in the selected language. Keep new-item intake sustainable when due work grows; the implemented default is up to 20 targets, due first, with at most five new targets filling remaining capacity. Early voluntary practice is still logged even when it does not promote the schedule.

These choices turn research into behaviour: accurate attempt grading, spaced returns after difficulty, meaningful cue coverage and use in varied prepared contexts. Software/content/device checks belong to implementation; no learner-run experiment or special delayed exam is part of the app.

## Reference audio and source intake

Preserve supplied course/actor audio and its exact phrase/dialogue-turn mapping. Generate missing clips during preparation, then store and replay them. Do not replace professional recordings simply to use one synthesis engine. Reference voice follows the phrase language and intended accent, never the UI/explanation language.

The requested [echo-words source inspection](../reference/echo-words-audio.md) provides this recommended implementation baseline:

| Language | Generation baseline from echo-words | Required handling |
|---|---|---|
| English | Piper `en_US-lessac-medium`; matching English Edge fallback | This voice is US English; respect a course's requested accent rather than silently substituting it. |
| German | Piper `de_DE-thorsten-medium` | An Edge fallback requires an explicitly configured German voice, not an English default. |
| Serbian | Edge `sr-RS-SophieNeural` | Convert Serbian Latin to Cyrillic for speech input only; keep displayed script unchanged. Do not use the misleadingly Serbian-labelled Lower Sorbian Piper voice identified in echo-words. |

Generate whole phrases/turns for natural delivery, not concatenated isolated word recordings. Review partial-cue audio separately so it cannot reveal hidden words. Preserve source/performer or engine/voice metadata, exact speech input, locale/accent, conversion/generation versions and source revisions. Include preparation settings in cache/job identity; preserve original course audio alongside any processed playback version.

Use only matching-language fallbacks. Failed generation remains an explicit retryable preparation job; existing prepared audio works during provider outages. Reuse echo-words' per-language approach, not its single-word Commons lookup, language/text-only cache identity, Anki lifecycle, exact 1 GB host limits or free-only budget rule. Its code/documentation were inspected; no new listening/quality benchmark was performed here.

Sources in scope are existing lesson/phrase notes with attachments and selected echo-words material. Start with reviewed manual packages; add source adapters and the LLM preparation pipeline in P5. Preserve stable source IDs, meaning, language, lesson membership and audio provenance; repeated imports must not duplicate targets or reset progress. Echo-words handoff needs durable export IDs, retryable delivery and exact phrase/context audio matching. A word-only clip cannot stand in for a complete phrase. Source files remain read-only unless separately authorized; no course enrollment or Anki migration follows from this plan.

## Proposed architecture and durable state

Use the existing Python scaffold. Recommended stack: Vue 3 + TypeScript/Vite mobile PWA, FastAPI/Pydantic, SQLAlchemy/Alembic and SQLite. Serve client/API from one origin with one API process and a bounded worker; local reference-media files and SQLite learner-audio BLOBs. See the [architecture](../architecture.md) and [contracts](../architecture/data-contracts.md).

Persist versioned targets, realizations/chunks, cue variants, lessons, prepared dialogues, explanations, source links, search metadata, reference assets, app settings, sessions, attempts, exposure, recordings, review states/receipts and preparation jobs. A content edit cannot silently rewrite an old attempt; changed meaning creates a new target.

The browser state machine owns display/recording/playback phases, local checkpoints and pending events/audio. The server owns published content, committed history and schedules. Commit attempts and eligible schedule updates atomically; use event IDs, sequence numbers and schedule versions for retries/stale sessions. One active scheduled session, explicit takeover and language-specific route views are separate concerns. Interrupted recording never resumes as a completed response.

Prepare new content through draft → structural checks → linguistic/cue review → published. Use bounded durable jobs with leases, explicit provider configuration and no implicit paid escalation. Prepared practice does not depend on live AI, speech recognition or synthesis. Initial human/content review and actual provider checks are implementer work, not an effectiveness study for the learner.

## Delivery sequence and feature coverage

| Phase | Work and concrete deliverable | Required coverage |
|---|---|---|
| **P1 — Interaction and device prototype** | Reviewed cue fixtures, runnable discussion prototype, real iPhone/desktop recording probe. | All language/navigation/card states above; original-attempt grading and separated-retry flow; actor/generated Reference audio; accessible compact controls. [Detailed P1](01-prototyping.md). |
| **P2 — Foundation, languages and content** | Local app in the existing scaffold; settings/routes/i18n, versioned content, manual imports, scoped library/search, reference playback. | Native vs learning vs UI language; bookmark/reload/two-tab behaviour; full lesson lists and concept lookup; actor provenance and correct text/audio. [Detailed P2](02-core-implementation.md#p21--application-skeleton-and-contracts). |
| **P3 — Complete core trainer** | Durable drill/explanation/answer flow, recordings, scheduling, repair, cue selection, dialogues and pause/resume. | One schedule/target; every take in DB from the first useful build; optional recording and silent self-check; support and original result survive retries/reload. [Detailed P3](02-core-implementation.md#p31--end-to-end-session). |
| **P4 — Phone and connection reliability** | Harden real-device capture/playback/storage and completion of downloaded sessions. | Interrupted capture, permissions, quota/reconnect, stale clients, language views, accessibility and recovery without lost takes or duplicate reviews. [Detailed P4](04-validation-and-release.md). |
| **P5 — Preparation and integrations** | Source adapters, designed/measured LLM phrase-preparation prompts, cue variants, explanations/search metadata, multilingual speech preparation and echo-words handoff. | Semantically reviewed masks that exercise different parts and are actually selected later; whole-phrase reference audio; explicit preparation failures/retry; idempotent intake preserving progress. [Detailed P5](03-content-and-integrations.md) and [prompt design](phrase-preparation.md). |
| **P6 — Private release and recovery** | Measured host choice, deployment runbook, backups, restore/upgrade/rollback evidence. | All-language content/progress/settings, every learner recording and reference assets survive restore; ordinary use requires no efficacy pilot. [Detailed P6](04-validation-and-release.md#p61--choose-and-prepare-the-host). |

P1 precedes the production foundation. P3 is an internal useful milestone with reviewed manual content, not completion of the agreed add-time LLM pipeline. P4 and P5 may proceed independently after the core is stable; the full planned feature set includes P5. Choose hosting in P6; a private development HTTPS endpoint for device checks does not select the production host. Do not mark a phase complete from simulated audio, an HTML illustration or an archived experiment.

## End-to-end acceptance checklist

These are software, content and interaction checks for the implementer, not research tasks for the learner. The phase documents specify additional failure cases.

- [ ] First launch persists native language. English and Serbian bookmarks open the right library after reload and in separate tabs. Both UI modes retain native-language explanations; changing languages preserves session/history.
- [ ] `rap` filters lesson titles; concept-only `greetings` finds stored phrases. Lookup → Show & explain → full lesson list → lesson drill works; browsing alone never advances a schedule.
- [ ] A known imported phrase drills directly. Forgotten/unclear material can open help before responding. The compact cue is sufficient, requests a whole-phrase response, and omits the rejected filler text.
- [ ] Mic → Stop recording → own take with cue → complete phrase/reference → self-check works on the actual phone. Replay/extra practice remain explicit and only one audio stream plays at a time.
- [ ] Recording-off practice requires no microphone, uses manual reference playback and never claims demonstrated spoken performance. Mode changes do not relabel prior takes.
- [ ] Show & explain keeps the full phrase visible during its recording/comparison flow. Its takes and lookup-origin takes are retained as supported practice without creating a fictitious due review.
- [ ] Every take, including retries and recoverable interrupted clips, survives session completion, restart and database restore. Lost acknowledgements create no duplicates; storage failure does not display Saved.
- [ ] Correct-but-effortful retrieval, failure, allowed equivalents, unclear cues and technical errors follow different outcomes. Extra pronunciation practice cannot change correct recall to failure. Accidental-rating correction remains deferred; no Undo UI is present.
- [ ] Help/exposure survives reload. Only the first eligible review-group result updates FSRS; retries, dialogue repetitions, two tabs/devices and request retransmission cannot inflate the interval.
- [ ] Failed targets return after intervening time/material, or a later session when too short. The bounded retry queue does not conflict with library steps, wait on a timer or loop forever.
- [ ] A later session can use another prepared mask of the same target, favouring missing/weak coverage. One reliable cue is reusable; ambiguous cues are withheld. Prepared context/dialogue variation preserves meaning and resumes a repaired dialogue correctly.
- [ ] The LLM preparation and review prompts have representative real-provider results with fresh semantic review. Useful variants are generated, checked, published and exercised through the actual selector; a nonempty cue or valid JSON alone does not satisfy preparation quality. Existing imported phrases can gain variants without resetting progress or rewriting pinned sessions.
- [ ] Actor audio is preserved. Generated English/German/Serbian clips speak the whole intended phrase with the configured voice; Serbian speech conversion leaves displayed text intact. Wrong-language fallback is prevented; failed preparation can retry without replacing good assets.
- [ ] Re-import and echo-words delivery retries preserve identities/progress. Existing prepared sessions remain usable while providers are unavailable; no live generation is needed for advancing.
- [ ] Accessibility, actual-device interruptions, downloaded-session recovery, private access and backup/rollback have recorded results before the corresponding release claim.

Record completed phase evidence in `specs/plan/evidence/`: build/version, device/browser when relevant, scenarios, results, unresolved limitations and links to artifacts. Do not put private learner recordings in reports or source control. P1 and P2 engineering closeout reports exist; their explicitly recorded device/content gaps remain open.

## Remaining choices and proposed defaults

| Choice | Current recommendation or next decision |
|---|---|
| Selected-lesson eligibility | Due/new, selected under user delegation on 2026-10-12; implemented in the session planner. |
| Exact answer labels, FSRS/repair/coverage details | Carry the concrete recommendation above into the prototype. Do not ask the learner to choose scientific parameters or validate effectiveness. |
| Recording on/off initial preference | Accepted on 2026-10-12: on, with explicit microphone start; preserve the mode on resume. |
| UI preference and language routes | Header selector, `/learn/{language}`, native/learning UI toggle initially off; exact layout/default remains proposed. Multi-language URLs themselves are required. |
| Final visual styling and explanation depth | Keep the agreed compactness; calm styling and expandable phrase breakdown are recommendations, not approved visual design. |
| Session length, new-item limits and empty due queue | Establish a usable flow in P1 without adding learning experiments. |
| Search implementation, initial content/translation inventory and source rollout | Meaning-aware search and multi-language use are required; choose implementation and delivery scope in P1/P5. |
| Platform, generation deployment and host | PWA is recommended; real-device and host/resource checks decide feasibility. Echo-words' machine constraints do not preselect the host. |

Do not reopen settled manual stop, automatic storage of every take, recording on Show & explain, the unified session entry or the need for separate language roles.

## Deferred, excluded and superseded

**Deferred:** automatic-stop detection and its app setting; historical recording-progress UI; automatic speech grading; locked-screen/eyes-free operation; arbitrary new offline sessions. Broader library/progress views and personal notes remain optional supporting proposals, not requirements to expand the first slice.

**Excluded or not authorized:** free conversation/role-play; learner-run validation, policy comparisons, calibration tasks or special exams; course enrollment; old Anki migration; automatic echo-words output switching; public deployment, Git initialization or source-repository changes merely because a future plan mentions them. Social features and grammar/reading courses are outside this trainer's scope.

**Superseded proposals:** separate Learn/Rehearse home modes, compulsory introductions for every DB item, Model as the audio-button label, Got it/Try again as the preferred grading design, random masks, selective/manual recording saving, the fixed review ladder/reset/ceiling and every-chunk-at-every-interval gate. Historical sketches do not reinstate them.

## Execution handoff and scaffold

P1, P2 and P3 core engineering are closed; review their evidence and the current handoff before beginning P4 or P5. The original execution order was **P1.1**: a small reviewed fixture pack, then the complete interaction prototype and real audio probe. Use the recommendations above to make unresolved behaviour concrete and reviewable; do not ask the user to restate already recorded requirements. Record walkthrough corrections as decisions before claiming visual approval or a passed gate.

Preserve `src/phrase_gym`, the rich-click entry point, hatchling/uv, pytest, Invoke, quality checks, documentation and CI conventions. Replace scaffold template behaviour only during authorized implementation. Keep all development context in `specs/`, plural. Do not install or extract the stopped temporary implementation as the starting app.

Run checks appropriate to implemented changes: controlled clocks for scheduling, transaction/retry tests for durability, browser tests for real flows, device tests for microphone/playback and actual-provider/content checks for generated material. Source inspection and mocked provider tests alone do not establish speech or cue quality. Future echo-words changes follow its own current instructions; this planning task changes no echo-words code or Anki data.

Follow the new session's actual user request. Updating this plan is not permission to start coding, deploy or claim a completed milestone.
