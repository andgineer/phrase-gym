# Delivery ladder

The adult learner's goal is to bring useful expressions from courses and videos into their own speech. Daily and Extra provide repeatable bounded phrase recall with factual practice results, a streak and a forgiving weekly target. The Coach field and its preparation path are in place; Coach's Claude preparation still needs a semantic review of held-out outputs and a real end-to-end run before content sources expand. The [practice and engagement decision](../decision-transfer-and-engagement.md) retains the behavior and evidence distinctions. Phrase recall is not freer speech; freer speech needs a distinct activity design before it can be scheduled or claimed.

Every stage preserves the learner's existing database and upgrades it in place under the [data compatibility rule](../architecture.md#releases-and-data-compatibility); stages use current import packages and need no support for older import formats.

## 4. Prepare typed phrases and words in Coach

**Learner outcome:** type a phrase or word into the header field, press **Coach** and later practice a complete card for it, without handling any file or JSON.

The [Coach field](../decision-external-preparation-agent.md#coach-field-typed-phrases-and-words), the Claude Code CLI adapter and the preparation path are in place, and the prompts are tuned on the development set. Remaining work needs actual Claude results:

- Freeze the prompts, run the untouched held-out items the orchestrator supplies, and have a fresh reviewer assess those outputs semantically.
- Make one real Claude run from the Coach field to a first rated attempt for a typed phrase and for a typed word, and one insufficient input that fails with its concrete reason.
- Listen, attended, to Coach's Piper references.

**Exit:** a typed English phrase and a typed word each go from the Coach field through actual Claude preparation to a first rated attempt, later productive recall and saved replay in the same app. Stated failures remain actionable and do not damage existing cards.

## 5. Extend only the content the learner next needs

**Learner outcome:** use the same workflow for another selected language or richer source material.

Choose one concrete need at a time: German chunks with Russian cues/help; Serbian after resolving help preferences; or richer source material. Retain the complete-example import format and [chunk content decision](../decision-targeted-drilling.md#phrases). Verify that language's comprehension, prompts and one voice separately. A slot system or full dialogues require a demonstrated learning use, not parity with the prototypes.

**Exit:** one representative batch in the selected extension works through the existing complete loop. No all-language/platform matrix gates delivery.

## 6. Add convenience after actual use

Choose improvements from observed friction: a settings editor for the stored daily due/new limits, a launcher, simpler export/import, an explicit Recreate action after internal preparation exists, or a button for the same stopped-app backup/restore. Keep the architecture small even in personal production use.

Qualify Codex as an alternative Coach CLI with real development and held-out outputs and a fresh semantic review, using the same frozen vendor-neutral prompts with no Codex-specific variant. This is not a prerequisite for Stage 5 or any other stage.

Implement the [per-user Coach service commands](../architecture.md#package-and-commands) after the local MVP: `install-coach` registers and starts Coach, and `uninstall-coach` removes the registration while preserving data. Verify startup and removal on macOS launchd and Linux systemd user services; neither command requires the cloud `deploy` extra.

Cloud placement of Phrase Gym with Coach on the learner's laptop is planned after the local MVP under the [access boundary](../architecture.md#local-access-and-privacy). The packaged `phrase-gym deploy` command must use a versioned installed artifact without a source clone, following the [package boundary](../architecture.md#package-and-commands). Design remote access, authentication and the cloud provider deployment before that work; actual deployment requires the learner's explicit request. Multi-device sync, automatic speech assessment, a live AI conversation partner, memorized full-dialogue rehearsal and broad distribution are not scheduled work. Add other stages only when a concrete user need justifies them.

After the MVP, qualify Linux browser media playback and recording under the [platform boundary](../architecture.md#application-boundary). Keep Linux server tests passing throughout; browser audio qualification is separate from server compatibility.

## Standing authorization for the local English demo

Existing authorization for routine local English preparation and Piper runs continues. Reuse the selected Claude subscription login, or Codex when explicitly selected, with the normal user home, existing voice cache and private attempt storage. No repeated budget, temporary authentication environment or login approval is needed. Do not copy tokens or read repository .env files. Recordings and application data stay outside model access.

Routine inspection of supplied external-AI material and local English synthesis is within the demo. Use the existing runtime bounds when running the prototype. The replacement needs bounded execution, limited output and explicit retry, not the old distributed accounting machinery. Do not bypass current counters or spin on quota/authentication failures.

The first MVP does not require a provider call. If actual model preparation is measured, inspect every output and retain separate fresh semantic review; normal configuration/skills may load and customization isolation is not claimed. Paid API fallback, provider switching, broader unrequested experiments, source mutation, login automation, native permission dialogs and deployment remain outside authorization. Repository pushes follow the [standing delivery authorization](../../AGENTS.md#git-secrets-and-deployment).

## Delivery discipline

Use the [implementation handout](orchestration-handout.md) and the repository [done gate](../../AGENTS.md#done-gate). Tests establish software behavior, semantic review establishes the inspected content, and learner interaction establishes actual-device acceptance. Report these separately. Remove accepted work from this ladder; do not accumulate completed-stage logs or let optional later stages redefine the MVP.
