# Delivery ladder

Deliver the smallest useful research-informed learning loop first. The target is one local Python application, SQLite and the existing Vue GUI. Both earlier backends supply reusable code and tests; neither defines mandatory parity. The [architecture](../architecture.md) excludes a separate native Coach, worker infrastructure and live recovery.

The adult learner's goal is to bring useful expressions from courses and videos into their own speech. Deliver recall first, then speaking transfer and sustainable engagement before expanding content sources. The [transfer decision](../decision-transfer-and-engagement.md) retains the behavior and evidence distinctions.

## Why phrases first

Import a batch of already prepared English course phrases from an external AI. This is the shortest route to practising personally useful language: it avoids building internal generation, echo-words integration and word-sense clarification before the recorder, recall loop and scheduled return are useful.

English matches the learner's resolved preferences and an existing local Piper candidate. This is a delivery choice, not evidence that phrases teach better than words. Source extraction stays outside this repository. A supplied export is sufficient; do not build a crawler or edit a source project.

## 1. First MVP: import a phrase batch and use it across days

**Learner outcome:** import 5–10 useful phrases once, study a few, return on a later day to say them from meaning, compare recordings, self-rate and see when they return.

### 1b. Later recall, scheduling and MVP completion

- Complete the [learning loop](../decision-core-trainer.md): later-day independent recall, cue rotation, reveal/compare, three ratings of the original response, one supported repetition where needed, due return and optional ungraded practice. Use library FSRS with the simplified eligibility policy.
- Add basic due/new counts, library lookup, Suspend/Restore and the minimal English language/timezone choices. Corrected imports can replace a suspended card without rewriting its history.
- Provide and verify the manual stopped-app backup/restore procedure. Preserve the cards, study history and recordings created in stage 1a.

**Exit:** satisfy the complete MVP acceptance table and checks below. A later real-clock return remains pending until observed; it need not block independent implementation work. A request for the whole first MVP covers both 1a and 1b.

### Observable acceptance

| Action or condition | What the learner must observe |
|---|---|
| Import a real externally prepared batch | Readable phrases, preparation progress and the confirmed added count; replayable references without developer seeding |
| Import malformed content or repeat the same package | An item-specific correction message or the existing result; no partial batch and no duplicate cards |
| Study a new phrase | Complete wording, understandable meaning, reference playback and Next; no success/failure grade |
| Finish today's study | A clear next-return indication; no forced waiting or immediate graded readback |
| Return on a later learning day | An answer-free cue and optional recording; varied cues across returns where available |
| Reveal and self-rate | Reference and short help; rating describes the original response, and one saved review changes the due date |
| Fail, ask to study, find an ambiguous card or encounter microphone failure | Corrective study, skip or suspension as appropriate; technical/card errors do not become forgetting grades |
| Record twice, reload and restart the app | Both complete saved takes remain associated with the right card and can be replayed |
| Practise early or repeat after rating | Clearly ungraded rehearsal; no second schedule update |
| Stop the app, back up, restore a copy and launch it | Cards, due history, references and saved takes are present and playable |

**Checks:** silent unit/integration and real-browser tests against the new Python backend cover this path and its simple failure/retry cases. Use a test clock for automated day/interval tests, never a learner-facing clock override. Review representative externally generated cards semantically and listen to references separately; software tests cannot establish either. The learner verifies complete microphone/comparison/saved playback on the actual MacBook. A later real-clock return remains explicitly pending until observed; immediate repetition is not a substitute.

**MVP boundary:** no word generation, frames, speaking missions, game rewards, dialogue runs, confusable dependency graph, multilingual voice matrix, native app, external inbox API, remote access or advanced backup system is required. A useful English phrase workflow is the first MVP, not a stepping stone that must wait for those features.

## 2. Bring learned expressions into freer speech

**Learner outcome:** use a few learned expressions while composing personally relevant speech, then practise related situations without supplied wording.

- Implement the [speaking progression](../decision-transfer-and-engagement.md#from-retrieval-to-freer-speech): supported personal responses, a repeated task with a changed situation or follow-up, and an unscripted attempt with no displayed target wording. Accept other natural responses; do not score compulsory phrase insertion.
- Reuse recording/replay and existing expressions. Import a small externally prepared set of situations and follow-ups first, including adaptable expressions demonstrated through complete examples. No live AI partner, slot editor or whole-dialogue engine is required.
- Add preparation for a real conversation and optional Used / No opportunity / Wanted it but could not retrieve it notes. Make a prompt set usable with a tutor or partner without requiring messaging, automatic sharing or a partner account.
- Keep activity/support records, unscripted attempts and real-use notes separate from card grades. Phrase exposure affects later independent-card eligibility; activities and usage reports do not update FSRS. Upgrade existing MVP data without losing cards, grades, audio or old-format import support.

**Exit:** the learner can complete a supported response, a changed unscripted task and an optional real-opportunity note, replay the saved takes and see which assistance was available. Tests cover state, exposure and scheduling separation, saved history and the upgrade path. Content and actual-use acceptance remain separate from software checks; no effectiveness experiment gates delivery.

## 3. Make adult practice sustainable

**Learner outcome:** choose short meaningful practice missions, see concrete progress and return after a break without penalties.

- Apply the [adult engagement decision](../decision-transfer-and-engagement.md#adult-engagement): chosen missions with varied situations, a clear finish and a learner-selected weekly target.
- Show dated recall progress, supported/unscripted practice, recording comparisons and real-use reports distinctly. Do not compute an invented fluency or mastery score.
- Add optional completion points or achievements, including credit for honest attempts that need correction. They must not depend on favorable self-ratings or reward endless easy-card repetition. Turning them off leaves all practice usable.
- Preserve accumulated progress across missed days and let the learner pause or adjust the target. No public leaderboard, mandatory collectibles, punitive streaks or virtual economy.

**Exit:** the learner can choose and finish a mission, see accurately labeled progress, disable rewards and return after a missed week without debt or lost progress. Behavioral tests cover these rules and prove that rewards cannot alter memory grades. Enjoyment and sustained participation remain distinct acceptance questions, not inferred learning gains.

## 4. Add echo-words through bounded internal preparation

**Learner outcome:** paste words with their source contexts or import a supplied echo-words export and receive useful word-sense cards without manually authoring them.

- At the start of Coach/internal preparation implementation, resolve the [Coach interface question](../open-problems.md): check whether Python with the existing web UI covers the required workflows without a native desktop GUI.
- Reuse the import, speech and practice path. Add a single selected subscription-CLI adapter, bounded execution and clear errors; retain the source material for explicit retry.
- Prepare one sense at a time, a comprehensible definition and two or three contexts under the [word rules](../decision-targeted-drilling.md#words-after-the-phrase-mvp). Ask briefly when meaning is genuinely ambiguous. Do not infer a sense from a bare homograph.
- Distinguish pending, ready and failed material. No distributed queue or second desktop application. Existing ready practice stays usable during generation.
- Document an export/paste interface first; direct echo-words integration or source changes require their own authorization. Do not make export automation a prerequisite.
- Use representative new actual model results with a fresh semantic reviewer, plus attended speech listening. Cover an ambiguous word, an accepted variant, a confusable contrast and a failed request separately from structural tests.

**Exit:** a learner-supplied English word batch goes from ordinary intake through preparation to first study, later productive recall and saved replay in the same app. Stated failures remain actionable and do not damage existing cards. No generalized provider framework is needed.

## 5. Extend only the content the learner next needs

**Learner outcome:** use the same workflow for another selected language or richer source material.

Choose one concrete need at a time: German chunks with Russian cues/help; Serbian after resolving help preferences; original course audio; or useful variable-expression examples. Verify that language's comprehension, prompts and one voice separately. A slot system or full dialogues require a demonstrated learning use, not parity with the prototypes.

**Exit:** one representative batch in the selected extension works through the existing complete loop. No all-language/platform matrix gates delivery.

## 6. Add convenience after actual use

Choose improvements from observed friction: a launcher, simpler export/import, an explicit Recreate action after internal preparation exists, or a button for the same stopped-app backup/restore. Keep the architecture small even in personal production use.

Remote hosting, multi-device sync, automatic speech assessment, a live AI conversation partner, memorized full-dialogue rehearsal and broad distribution are not scheduled work. The speaking tasks above do not depend on them. Add a new stage only when a concrete user need justifies it.

## Standing authorization for the local English demo

Existing authorization for routine local English preparation and Piper runs continues. Reuse the selected Codex or Claude subscription login, normal user home, existing voice cache and private attempt storage. No repeated budget, temporary authentication environment or login approval is needed. Codex uses this repository as its workspace; do not copy tokens or read repository .env files. Recordings and application data stay outside model access.

Routine inspection of supplied external-AI material and local English synthesis is within the demo. Use the existing runtime bounds when running the prototype. The replacement needs bounded execution, limited output and explicit retry, not the old distributed accounting machinery. Do not bypass current counters or spin on quota/authentication failures.

The first MVP does not require a provider call. If actual model preparation is measured, inspect every output and retain separate fresh semantic review; normal configuration/skills may load and customization isolation is not claimed. Paid API fallback, provider switching, broader unrequested experiments, source mutation, login automation, native permission dialogs, push and deployment remain outside authorization. Preserve existing learner data; prototype code retirement does not authorize deleting it.

## Delivery discipline

Use the [implementation handout](orchestration-handout.md) and the repository [done gate](../../AGENTS.md#done-gate). Tests establish software behavior, semantic review establishes the inspected content, and learner interaction establishes actual-device acceptance. Report these separately. Remove accepted work from this ladder; do not accumulate completed-stage logs or let optional later stages redefine the MVP.
