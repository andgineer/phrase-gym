# Delivery ladder

The adult learner's goal is to bring useful expressions from courses and videos into their own speech. Daily and Extra provide repeatable bounded phrase recall with factual practice results and a streak; next, make practice sustainable before expanding content sources. The [practice and engagement decision](../decision-transfer-and-engagement.md) retains the behavior and evidence distinctions. Phrase recall is not freer speech; freer speech needs a distinct activity design before it can be scheduled or claimed.

Before the first release, every stage uses a freshly created application database and current import packages. No pre-release stage requires a database migration or support for older import formats.

## 3. Make adult practice sustainable

**Learner outcome:** choose short meaningful practice missions, see concrete progress and return after a break without penalties.

- Apply the [adult engagement decision](../decision-transfer-and-engagement.md#adult-engagement): chosen missions with varied situations, a clear finish and a learner-selected weekly target.
- Build on the factual [practice facts](../decision-transfer-and-engagement.md#practice-entry-and-todays-facts) and saved recording comparisons, each distinctly labeled. Do not compute an invented fluency or mastery score.
- Add optional completion points or achievements, including credit for honest attempts that need correction. They must not depend on favorable self-ratings or reward endless easy-card repetition. Turning them off leaves all practice usable.
- Preserve accumulated progress across missed days and let the learner pause or adjust the target. Keep the current/best practice streak factual and separate from rewards and memory grades. No public leaderboard, mandatory collectibles, streak-based penalty or virtual economy.

**Exit:** the learner can choose and finish a mission, see accurately labeled progress, disable rewards and return after a missed week without debt or lost progress. Behavioral tests cover these rules and prove that rewards cannot alter memory grades. Enjoyment and sustained participation remain distinct acceptance questions, not inferred learning gains.

## 4. Add echo-words through bounded internal preparation

**Learner outcome:** paste words with their source contexts or import a supplied echo-words export and receive useful word-sense cards without manually authoring them.

- Reuse the Phrase Gym submission and publication path with Coach performing preparation. Add a single selected subscription-CLI adapter in Coach, bounded execution and clear errors; retain the source material for explicit retry.
- Prepare one sense at a time, a comprehensible definition and two or three contexts under the [word rules](../decision-targeted-drilling.md#words-after-the-phrase-mvp). Ask briefly when meaning is genuinely ambiguous. Do not infer a sense from a bare homograph.
- Distinguish pending, ready and failed material. Reuse the small durable Coach handoff; no general distributed queue or second desktop application. Existing ready practice stays usable during generation.
- Document an export/paste interface first; direct echo-words integration or source changes require their own authorization. Do not make export automation a prerequisite.
- Use representative new actual model results with a fresh semantic reviewer, plus attended speech listening. Cover an ambiguous word, an accepted variant, a confusable contrast and a failed request separately from structural tests.

**Exit:** a learner-supplied English word batch goes from ordinary intake through preparation to first study, later productive recall and saved replay in the same app. Stated failures remain actionable and do not damage existing cards. No generalized provider framework is needed.

## 5. Extend only the content the learner next needs

**Learner outcome:** use the same workflow for another selected language or richer source material.

Choose one concrete need at a time: German chunks with Russian cues/help; Serbian after resolving help preferences; or richer source material. Retain the complete-example import format and [chunk content decision](../decision-targeted-drilling.md#phrases). Verify that language's comprehension, prompts and one voice separately. A slot system or full dialogues require a demonstrated learning use, not parity with the prototypes.

**Exit:** one representative batch in the selected extension works through the existing complete loop. No all-language/platform matrix gates delivery.

## 6. Add convenience after actual use

Choose improvements from observed friction: a settings editor for the stored daily due/new limits, a launcher, simpler export/import, an explicit Recreate action after internal preparation exists, or a button for the same stopped-app backup/restore. Keep the architecture small even in personal production use.

Implement the [per-user Coach service commands](../architecture.md#package-and-commands) after the local MVP: `install-coach` registers and starts Coach, and `uninstall-coach` removes the registration while preserving data. Verify startup and removal on macOS launchd and Linux systemd user services; neither command requires the cloud `deploy` extra.

Cloud placement of Phrase Gym with Coach on the learner's laptop is planned after the local MVP under the [access boundary](../architecture.md#local-access-and-privacy). The packaged `phrase-gym deploy` command must use a versioned installed artifact without a source clone, following the [package boundary](../architecture.md#package-and-commands). Design remote access, authentication and the cloud provider deployment before that work; actual deployment requires the learner's explicit request. Multi-device sync, automatic speech assessment, a live AI conversation partner, memorized full-dialogue rehearsal and broad distribution are not scheduled work. Add other stages only when a concrete user need justifies them.

After the MVP, qualify Linux browser media playback and recording under the [platform boundary](../architecture.md#application-boundary). Keep Linux server tests passing throughout; browser audio qualification is separate from server compatibility.

## Standing authorization for the local English demo

Existing authorization for routine local English preparation and Piper runs continues. Reuse the selected Codex or Claude subscription login, normal user home, existing voice cache and private attempt storage. No repeated budget, temporary authentication environment or login approval is needed. Codex uses this repository as its workspace; do not copy tokens or read repository .env files. Recordings and application data stay outside model access.

Routine inspection of supplied external-AI material and local English synthesis is within the demo. Use the existing runtime bounds when running the prototype. The replacement needs bounded execution, limited output and explicit retry, not the old distributed accounting machinery. Do not bypass current counters or spin on quota/authentication failures.

The first MVP does not require a provider call. If actual model preparation is measured, inspect every output and retain separate fresh semantic review; normal configuration/skills may load and customization isolation is not claimed. Paid API fallback, provider switching, broader unrequested experiments, source mutation, login automation, native permission dialogs and deployment remain outside authorization. Repository pushes follow the [standing delivery authorization](../../AGENTS.md#git-secrets-and-deployment).

## Delivery discipline

Use the [implementation handout](orchestration-handout.md) and the repository [done gate](../../AGENTS.md#done-gate). Tests establish software behavior, semantic review establishes the inspected content, and learner interaction establishes actual-device acceptance. Report these separately. Remove accepted work from this ladder; do not accumulate completed-stage logs or let optional later stages redefine the MVP.
