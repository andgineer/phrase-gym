# P1 — prototypes and validation

Status: **no approved deliverables; gate not passed**, 2026-10-11. The stopped temporary implementation is archived and does not count as completion. The user requires design discussion followed by implementation in a new session. Dependencies: [agreed targeted drilling](../decision-targeted-drilling.md). Outcome: evidence and a concrete design to review before production implementation. Earlier in-conversation screens and legacy code do not count as these prototypes.

Coverage comes from the [consolidated implementation plan](implementation-plan.md), especially its agreed behaviour, current recommendations and end-to-end checklist. The original seven walkthroughs below are the practice core, not permission to omit the subsequently added home/search, language URLs, UI-language separation or revised answer states. Use current Reference / My recording wording when revising earlier sketches.

## P1.1 — Content and cue prototype

Prepare a compact fixture pack: initially about 12 targets, including fixed expressions, words in phrases and reusable frames; two short specific dialogues; two or more reviewed cue variants where useful. The size is a manageable test set, not a pedagogical prescription.

For every target, write its complete wording, purpose, selected meaning, allowed variants/slots, meaningful chunks, model audio source and the distinction between planned cues and rescue help. Include visible anchors from the middle/end as well as the beginning. For each mask, list plausible competing completions and adjust the cue until its requested expression is clear in the learned set/context.

Incorporate the [research-to-design changes](../design/answer-card-and-review-policy.md#what-the-research-changes-in-the-design): cues covering complementary/weak chunks with comparable support, suitable different prepared situations for one target, and concise meaning/use help. Include a fixture where every prior cue supplied an important chunk so the selector's coverage behaviour can be inspected.

Include deliberate counterexamples: a broad situation with multiple valid answers, a cue that reveals the target word, an unfamiliar expression, a long phrase, a contraction, inflection within an allowed frame and an ambiguous gap. Bad candidates stay in rejection fixtures, not the practice queue.

**Deliverables:** versioned fixture JSON plus a short human-readable cue review sheet. AI may draft candidates during this task, but initial material is reviewed before use. No runtime generator is needed.

**Acceptance:** each practice cue has a clear task and rationale; the learner is never required to guess an unintroduced expression; rotating masks retain identity; the response contract is the whole phrase, without restoring the rejected repeated instruction on the drill card; word-use practice is distinguished from recalling that word. Record what remains ambiguous instead of claiming all-language uniqueness.

## P1.2 — Interaction prototype

After implementation is requested in the new session, build a disposable mobile-first prototype from P1.1 and the [discussed interaction design](../design/interaction-design.md). Use local fixtures and explicit temporary session state. Its purpose is to let the user inspect what appears and what they do at every moment, before committing to navigation, visual style or production APIs.

Required walkthroughs:

1. Forgotten or unclear source phrase: drill → explicit explanation action → complete phrase and explanation in the learner's language → return/practice with exposure retained → response and comparison. Also show that a newly imported but already familiar phrase can be drilled without a compulsory introduction. All paths share one Start session entry.
2. Known phrase: reviewed cue → response → self-check → another target → return with a different hidden chunk. Also demonstrate a later session selecting a different stored cue when a suitable alternative exists, plus reuse for a phrase with only one reliable cue. Fixtures stand in for the add-time LLM preparation pipeline at this prototype stage.
3. Stuck learner: additional meaningful help or the full phrase/explanation card → supported repair → later retry, with support retained in the result. Opening explanation alone is not a failure grade; returning to the drill hides learner-language help.
4. Word/frame: fixed learning objective with permitted substitutions; a shown target word is recorded as use, not word recall.
5. Different wording: distinguish practising the selected target from reporting an unclear cue; neither implies that valid English is wrong.
6. Specific dialogue: introduce the exchange → partner turn → cued learner turn → supporting line → isolate a hard turn and return to the exchange.
7. Pause/exit/reload: show what is resumed and what help has already been used.

Use the current proposed Not yet / With effort / Recalled answer card in these walkthroughs. Demonstrate correct-but-effortful retrieval, a genuinely missed target, pronunciation practice after correct recall, and help before any attempt. The original grade survives retries; silent retrieval supplies memory evidence only. Show the separated-retry queue with simulated time for engineering checks, without making the user wait or run a learning experiment.

Use the discussed one-expression screen with context above the cue as the recommended starting proposal. Final visual style is still open; neither old mockup is preselected or approved. Compare help placement, answer checking and difficulty reporting with the user. Include a small session result with one entry per target, not separate card statistics per mask.

Show a visible explanation action from the drill, including before any response, and full phrase plus native-language meaning/use on the explanation card. Native-language explanation must disappear from the ordinary drill on return while exposure remains recorded. Also validate the first-launch native-language question with an empty database and reuse of the persisted setting on a later launch. There is one Start session entry, not separate Learn new material/Rehearse modes.

The explanation-card walkthrough also includes optional practice recording: keep the full phrase visible through capture, automatic learner playback and model playback; allow replay/another take and return to the drill. Apply the session recording switch on both cards and keep the recording-off explanation path microphone-free. Verify that these takes are model-supported practice on the same target, without changing the earlier drill attempt's evidence.

Include the accepted [home/lookup/lesson routes](../decision-session-navigation-and-search.md): default All lessons under spaced repetition; lesson title filtering (`rap`); literal and concept search (`greetings` absent from matching phrase wording); result → Show & explain → lesson's complete phrase list → Start lesson drill. Cover no matches, returning with the query retained and opening phrases from the list. Lookup/list browsing is exposure, not a completed review or a due-date change. Settle selected-lesson due-only versus all-phrase policy separately; fixture ordering must not decide it implicitly. Validate real retrieval quality separately from a hand-authored sketch index.

Add the [language/audio requirements](../decision-languages-and-audio-labels.md): a header learning-language selector, URL/bookmark restoration for English and Serbian, independent language views in two tabs, and language-scoped lessons/search/reviews. Demonstrate both native-language UI and UI following the selected learning language while explanations stay native. UI controls may be native-language even though drill content stays target-language. Include switching away from a paused session without losing its history. Use Reference / My recording controls with both imported actor audio and a generated example; no label implies AI origin. Preference control/default and route syntax remain proposals to illustrate.

**Deliverables:** runnable prototype under `prototypes/drill-flow/`, fixture set and observed usability findings. It can be discarded; its existence is not production readiness.

**Acceptance:** the user can complete each walkthrough without the agent explaining what to say or which button to press. Record approval/corrections to the flow, target-specific cue examples and terms used for outcomes. Validate 320px layouts, touch targets, text scaling and keyboard access. An untested mockup is not a passed gate.

## P1.3 — Real-device audio and storage probe

Build a small standalone probe under `prototypes/audio-probe/`. Test on the user's actual iPhone in Safari and as an installed home-screen app, plus a desktop reference browser. Provide a development HTTPS origin on a private route; do not choose a production host or publish publicly.

| Probe | Evidence to collect |
|---|---|
| Reference playback | Imported actor/course and generated clips; starts after an explicit gesture; replay/stop/seek behaviour; speaker and headphones. |
| Recording and comparison | Session on/off switch, permission flow, supported MIME types, real playable recording, automatic learner-take playback followed by model reveal/playback, replay either version and rerecord. |
| Silent practice | Recording off needs no microphone; Reveal & compare works; model audio is manual; no spoken-performance credit is inferred. |
| Interruptions | Permission ignored/denied, incoming interruption, background/foreground, locked screen and headphone disconnect. Record actual limits. |
| Recovery | Reload between steps; incomplete recording is not presented as complete; supported practice still works when recording fails. |
| Prepared assets | A small downloaded session plays after disconnecting; missing asset produces a clear recovery path. |
| Local writes | Attempt journal and every captured take pending database storage survive normal restart; quota failure is handled. No manual Save step. Do not simulate persistence by merely retaining an in-memory object. End-to-end database retention is implemented in P3 and hardened in P4. |

Detect formats rather than assuming WebM or MP4 across devices. An audio-only request must not request camera permission. Stop microphone tracks when recording ends or the app exits the recording state. Do not upload probe recordings to any third-party model; delete disposable test takes through an explicit local control.

The required technical facts are in the [architecture's audio section](../architecture.md#audio-and-recordings). Browser emulation can verify state handling; it cannot substitute for the actual iPhone microphone, headphones or interruption tests.

**Deliverable:** capability report with OS/browser version, encoding and each result, including failures and untested cases. If the phone is unavailable, finish the probe and desktop checks, leave device acceptance pending, and do not report it passed.

## P1.4 — Decide the production starting point

Use the prototypes to finalize:

- the session flow, layout, help levels and outcome wording;
- what counts as a reviewed cue and which chunks constitute the target;
- whether the proposed review-policy eligibility and coverage rules are understandable;
- whether foreground PWA audio meets the required first-release use, or a native client must be considered;
- recording on/off defaults, first languages, and the first source/intake milestone. Retention is decided: automatically keep all learner audio in the database; historical-comparison UI is deferred.
- selected-lesson due-only versus all-phrase behaviour, session size/empty-queue handling, and the concrete native/learning UI preference and language-switch/resume presentation. Use the master plan's proposed routes/defaults to make these reviewable; multi-language URLs and separate explanation/UI roles are already required.

Short prototype use establishes clarity and technical feasibility, not long-term learning effectiveness. P6 covers ordinary use, reliability and issue correction. The user explicitly rejected delayed effectiveness pilots, special assessment sessions and calibration tasks; none is required to choose or use the recommended method.

**Exit gate:** a concrete prototype reviewed by the user, an honest device report, agreed changes recorded in decision documents, and an updated architecture/implementation scope. Any unresolved device capability remains an explicit release dependency. Only then begin P2.

## Latest interaction requirement

The [recording decision](../decision-recording-and-comparison.md) is part of P1's scope. Demonstrate the recording-on sequence without additional playback taps after finishing the take, plus interruption and playback-failure recovery. Keep the initial cue visible during the learner replay; reveal the full target for model comparison afterward. Use the accepted manual Stop recording. Automatic stopping and the app-settings choice of stopping method are deferred beyond the first version. Demonstrate recording-off silent practice as a complete session path and preserve mode changes and support on reload.

Final visual style and recording on/off defaults remain design decisions. Manual stopping and retention of all recordings in the database are resolved; do not reopen them as first-version blockers. Historical progress-comparison UI is deferred. Do not treat a text-only implementation or a passing reducer suite as completion of this interaction/audio gate.
