# Phrase Gym — current state and open choices

Updated 2026-10-11.

## Baseline and status

- **Agreed:** [targeted drilling](../decision-targeted-drilling.md), the name Phrase Gym, and [optional recording with automated comparison](../decision-recording-and-comparison.md).
- **Clarified by the user:** design discussion and a written plan come before implementation in a new session. No approved prototype exists. Earlier sketches, legacy files and the stopped temporary implementation do not complete any gate.
- **Current work:** the user resumed prototype and functionality discussion on 2026-10-11 and explicitly required all decisions to be preserved in `specs/`. Use the [current discussion record](../design/interaction-design.md#resumed-prototype-discussion-2026-10-11); update the relevant decision documents when choices are accepted. The scaffold is unchanged; all implementation phase gates remain unpassed. Read [START-HERE](../START-HERE.md) and the [next-session brief](next-session.md).

Start with the [implementation roadmap](implementation-plan.md) and [P1 prototyping](01-prototyping.md). The [architecture](../architecture.md) recommends a mobile PWA, small Python backend and durable target-level storage, subject to prototype findings. Detailed contracts cover [learning behaviour](../architecture/learning-engine.md) and [data/API boundaries](../architecture/data-contracts.md).

The [functional description](../functional-description.md) remains a draft of supporting scope. [Interaction design](../design/interaction-design.md) records the discussed screens and recording flow; the final visual layout is not approved. The [research record](../research-memorizing-phrases.md) distinguishes evidence from product choices.

## Choices to resolve through the plan

| Choice | Proposed starting point | Evidence/decision point |
|---|---|---|
| Exact session interface | Prototype complete user-visible flows from the agreed decision. | P1 user walkthrough; no inherited screen approval. |
| Session entry and explanation access — agreed; layout open | One Start session button, no separate learning/rehearsal entry modes. Expect prior source exposure, but provide full phrase and explanation in the learner's language directly from every drill. | Current drill/explanation-card sketch. Normal drills remain in the target language; labels, depth and return behaviour need review. |
| Native language — agreed; first-launch layout open | Ask on first launch with an empty database and store the native language in app settings for explanations. Reuse on later launches. | P1/P2. Russian is selected for the current sketch; production asks the learner. |
| Cue preparation — agreed; exact selection policy open | LLM prepares alternate hidden-word/chunk versions when adding a phrase to the database, where useful alternatives exist. Store checked variants for different presentations across sessions. Deterministic rotation remains the proposed selector. | P1 cue pack and cross-session walkthrough; P5 add-time preparation and generated-content evaluation. Do not force multiple cues for an unsuitable phrase. |
| Review eligibility and intervals | One schedule per target, explicit support/coverage, replaceable conservative policy. | P1 outcome review, P3 correctness tests and P6 delayed pilot. |
| Client platform | Mobile PWA is proposed; session recording and automated comparison are agreed. | P1 actual-phone audio probe; P4 reliability checks. |
| Recording stop mechanism — resolved | Manual Stop recording is the accepted default and the only first-version stopping method. | A manual/automatic choice belongs in later app settings. The selector and automatic stopping are deferred beyond the first version. |
| Visual style | Calm, focused screen recommended; stronger gym-like styling offered as an alternative. Neither selected. | Design discussion and P1 walkthrough. |
| First inputs | Phrases and word/frame targets share manual import; live echo-words handoff follows the core. | Confirm first-release scope in P1; implement P5. |
| Recording retention — resolved | Automatically store all recordings' audio and metadata in the app database from the first version, including retries; no manual Save or automatic expiry. | P3 persistence; P4 recovery validation. Historical progress-comparison UI is deferred. |
| Languages and supporting features | Explicit language metadata from the start; choose initial content languages, recording on/off defaults and help controls. | P1/P4. |
| Host and access | Private HTTPS; measure the named Oracle VM and laptop options. | P6, after there is a build to measure. |

Automatic speech grading, eyes-free/locked-screen operation and old Anki migration are separate possible additions. Open-ended AI conversation and free role-play remain excluded entirely. Additional course enrolment is not authorized and is not needed for these plans.
