# P1 — prototypes and validation

Status: **no approved deliverables; gate not passed**, 2026-10-11. The stopped temporary implementation is archived and does not count as completion. The user requires design discussion followed by implementation in a new session. Dependencies: [agreed targeted drilling](../decision-targeted-drilling.md). Outcome: evidence and a concrete design to review before production implementation. Earlier in-conversation screens and legacy code do not count as these prototypes.

## P1.1 — Content and cue prototype

Prepare a compact fixture pack: initially about 12 targets, including fixed expressions, words in phrases and reusable frames; two short specific dialogues; two or more reviewed cue variants where useful. The size is a manageable test set, not a pedagogical prescription.

For every target, write its complete wording, purpose, selected meaning, allowed variants/slots, meaningful chunks, model audio source and the distinction between planned cues and rescue help. Include visible anchors from the middle/end as well as the beginning. For each mask, list plausible competing completions and adjust the cue until its requested expression is clear in the learned set/context.

Include deliberate counterexamples: a broad situation with multiple valid answers, a cue that reveals the target word, an unfamiliar expression, a long phrase, a contraction, inflection within an allowed frame and an ambiguous gap. Bad candidates stay in rejection fixtures, not the practice queue.

**Deliverables:** versioned fixture JSON plus a short human-readable cue review sheet. AI may draft candidates during this task, but initial material is reviewed before use. No runtime generator is needed.

**Acceptance:** each practice cue has a clear task and rationale; the learner is never required to guess an unintroduced expression; rotating masks retain identity; full-sentence output is explicit; word-use practice is distinguished from recalling that word. Record what remains ambiguous instead of claiming all-language uniqueness.

## P1.2 — Interaction prototype

After implementation is requested in the new session, build a disposable mobile-first prototype from P1.1 and the [discussed interaction design](../design/interaction-design.md). Use local fixtures and explicit temporary session state. Its purpose is to let the user inspect what appears and what they do at every moment, before committing to navigation, visual style or production APIs.

Required walkthroughs:

1. New phrase: complete model and purpose → guided recall → complete spoken response → comparison.
2. Known phrase: reviewed cue → response → self-check → another target → return with a different hidden chunk.
3. Stuck learner: additional meaningful help/full model → spoken repair → later retry, with support retained in the result.
4. Word/frame: fixed learning objective with permitted substitutions; a shown target word is recorded as use, not word recall.
5. Different wording: distinguish practising the selected target from reporting an unclear cue; neither implies that valid English is wrong.
6. Specific dialogue: introduce the exchange → partner turn → cued learner turn → supporting line → isolate a hard turn and return to the exchange.
7. Pause/exit/reload: show what is resumed and what help has already been used.

Use the discussed one-expression screen with context above the cue as the recommended starting proposal. Final visual style is still open; neither old mockup is preselected or approved. Compare help placement, answer checking and difficulty reporting with the user. Include a small session result with one entry per target, not separate card statistics per mask.

**Deliverables:** runnable prototype under `prototypes/drill-flow/`, fixture set and observed usability findings. It can be discarded; its existence is not production readiness.

**Acceptance:** the user can complete each walkthrough without the agent explaining what to say or which button to press. Record approval/corrections to the flow, target-specific cue examples and terms used for outcomes. Validate 320px layouts, touch targets, text scaling and keyboard access. An untested mockup is not a passed gate.

## P1.3 — Real-device audio and storage probe

Build a small standalone probe under `prototypes/audio-probe/`. Test on the user's actual iPhone in Safari and as an installed home-screen app, plus a desktop reference browser. Provide a development HTTPS origin on a private route; do not choose a production host or publish publicly.

| Probe | Evidence to collect |
|---|---|
| Model playback | Starts after an explicit gesture; replay/stop/seek behaviour; speaker and headphones. |
| Recording and comparison | Session on/off switch, permission flow, supported MIME types, real playable recording, automatic learner-take playback followed by model reveal/playback, replay either version and rerecord. |
| Silent practice | Recording off needs no microphone; Reveal & compare works; model audio is manual; no spoken-performance credit is inferred. |
| Interruptions | Permission ignored/denied, incoming interruption, background/foreground, locked screen and headphone disconnect. Record actual limits. |
| Recovery | Reload between steps; incomplete recording is not presented as complete; supported practice still works when recording fails. |
| Prepared assets | A small downloaded session plays after disconnecting; missing asset produces a clear recovery path. |
| Local writes | Attempt journal and saved take survive normal restart; quota failure is handled. Do not simulate persistence by merely retaining an in-memory object. |

Detect formats rather than assuming WebM or MP4 across devices. An audio-only request must not request camera permission. Stop microphone tracks when recording ends or the app exits the recording state. Do not upload probe recordings to any third-party model; delete disposable test takes through an explicit local control.

The required technical facts are in the [architecture's audio section](../architecture.md#audio-and-recordings). Browser emulation can verify state handling; it cannot substitute for the actual iPhone microphone, headphones or interruption tests.

**Deliverable:** capability report with OS/browser version, encoding and each result, including failures and untested cases. If the phone is unavailable, finish the probe and desktop checks, leave device acceptance pending, and do not report it passed.

## P1.4 — Decide the production starting point

Use the prototypes to finalize:

- the session flow, layout, help levels and outcome wording;
- what counts as a reviewed cue and which chunks constitute the target;
- whether the proposed review-policy eligibility and coverage rules are understandable;
- whether foreground PWA audio meets the required first-release use, or a native client must be considered;
- recording defaults/retention, first languages, and the first source/intake milestone.

Short prototype use can establish clarity and technical feasibility. It cannot demonstrate long-term retention or automatic real-world use. Plan delayed pilot observations in P6; do not market a successful walkthrough as a learning-effectiveness study.

**Exit gate:** a concrete prototype reviewed by the user, an honest device report, agreed changes recorded in decision documents, and an updated architecture/implementation scope. Any unresolved device capability remains an explicit release dependency. Only then begin P2.

## Latest interaction requirement

The [recording decision](../decision-recording-and-comparison.md) is part of P1's scope. Demonstrate the recording-on sequence without additional playback taps after finishing the take, plus interruption and playback-failure recovery. Keep the initial cue visible during the learner replay; reveal the full target for model comparison afterward. Automatic stop on silence is not an accepted requirement. Demonstrate recording-off silent practice as a complete session path and preserve mode changes and support on reload.

Final visual style, stopping mechanism and retention/defaults remain design decisions. Do not treat a text-only implementation or a passing reducer suite as completion of this interaction/audio gate.
