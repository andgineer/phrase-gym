# Remaining implementation work

This plan contains unfinished work only. Remove entries as they are completed; lasting constraints belong in the linked decisions, and code/tests describe delivered behavior.

## External-agent intake and reference speech

Follow the [external-agent boundary](../decision-external-preparation-agent.md). The runtime still contains source parsing, application text-LLM preparation/review and special handoff workflows that do not belong in the app.

- Remove vault/legacy-note parsers, text-provider configuration/calls, generic LLM jobs/review APIs and source-specific handoff interfaces. Preserve data, migrations, generic validation/publication, immutable revisions, chunk reconciliation, media storage and reusable speech adapters. Remove tests for discarded responsibilities and refresh generated API contracts.
- Deliver a source-independent preparation/submission guide and the actual versioned schema. Include stable identities, meaning/chunk/cue/help/dialogue rules, optional-media upload/binding, validation feedback, idempotent retries and compatible updates. Document real commands/endpoints once implemented.
- Provide one generic prepared-material intake path without application linguistic review or operator approval. Verify unrelated sources, identical-text/different-meaning targets, retries, compatible updates and supplied-audio preservation. Demonstrate prepared prompts through actual rotation and pinned reload without resetting progress.
- Isolate missing-reference speech into focused bounded audio work with per-language primary/fallback and complete-phrase caching. Verify failure/retry, partial resume and cache invalidation without replacing good supplied assets. Language-specific voice selection/listening remains open.
- Assess the guide using representative externally prepared material and held-out examples: short replies, openers that share their first words, long phrases, near-duplicates across lessons, idioms, frames, contractions, missing context, German chunks, Serbian and each language's help setting. Demonstrate several accepted prompts per phrase and a phrase whose prompts all fail the whole-set check (study and lookup only). Inspect semantic quality separately from structural conformance; provider measurements need a planned fixture set and a fresh reviewer.

External sources remain read-only unless separately authorized. Preparation/provider choice and delivery retries belong to the external agent.

## Prompt and practice redesign

Implements the [prompts, words, help and practice decisions](../decision-targeted-drilling.md).

- **Prompt variants:** replace gapped-phrase cues with prompt variants (situation, optional preceding line with its audio, ordered hint words) across contracts, validation, publication, pinned revisions and generated client types. Re-prepare the local 44-phrase pack rather than converting its gapped cues. Rotation follows the [learning policy](../decision-core-trainer.md).
- **Card:**
  - Show the situation, and play and show the preceding line once. Show no phrase text before the response.
  - Hint reveals the hint words one at a time and marks the response supported.
  - The comparison ends with the reference and an ungraded "say it with the reference" step.
  - Browser tests cover hint support, rotation and that no phrase text appears before the response.
- **First meeting:** before a phrase's first recall, play the reference, show the phrase with its situation and ask for two say-alongs, with no grade. The first recall comes after the existing repair spacing (five minutes and two other finalized targets), or in a later session.
- **Help per language:**
  - Replace the single native-language explanation source with a help language and a translation-with-answer choice per learning language. English: English help, off. German: Russian, on, shown on reveal and after comparison. Serbian: not Russian; its help language is still open.
  - Keep the UI-language preference, and migrate existing settings.
- **Word items:**
  - Add a per-language level setting (beginner or independent) that selects the card format: English independent, German beginner.
  - Independent format: definition, rotating contexts, contrast notes, missing-word prompt with whole-sentence answer.
  - Beginner format: rotating whole chunks, understanding before saying, translation prompts for saying, gloss with article/plural on tap.
  - Both formats: one schedule per word, no two new confusables in one session, and mixed contrast drills once both are known.
  - Define when an item moves from understanding to saying.
  - Ask the user for Serbian's level and help language before preparing Serbian words.
- **Game elements:** dialogue runs with shrinking pauses and hint-free levels, the per-lesson count of phrases recalled without help, real-call phrase choice with marking afterwards, and a weekly target.
- **Preparation instructions:**
  - Rewrite the generator and reviewer instructions for prompt variants. Check each prompt against the learner's whole phrase set in that language, and produce distinguishing hints and help only where useful.
  - The external agent produces a review page where the learner accepts or rejects each prompt.
  - Measure on the representative fixtures above with a fresh reviewer before using the result for imports.
- **English vault import:**
  - Scope: 63 phrases in four lessons of `languages/english/фразы`, read-only at a recorded revision.
  - Build the package from accepted prompts. Take the other person's lines from the vault dialogues, reusing their existing audio. Add English notes only where useful, and no Russian.
  - Run a dry run against the real database, then import, reconcile media and verify playback after the user approves.

## Device acceptance

Complete the [device gaps](../open-problems.md) on actual iPhone Safari over trusted HTTPS, with speaker and headphones. Check longer/successive takes, saved replay, interruption/reopening, temporary server failure/retry and accessibility. Record only actionable gaps and current acceptance limits, without accumulating review logs in specs.

## Private release and recovery

- Measure the built app and bounded speech work on the proposed Oracle VM or always-on laptop: idle/API/media peaks, disk growth and impact on neighboring services. Do not assume speech models fit a shared 1 GB host.
- Prepare a concrete host/access/backup proposal and operational documentation covering configuration, start/stop/update, health checks, storage/logging and recovery. Host choice and deployment require an explicit user request.
- Demonstrate a consistent database/media backup and isolated restore, including every language, settings, schedules/history, every learner take, reference provenance and unfinished audio work. Rehearse upgrade and rollback using matching snapshots where needed.
- Verify actual private access and health from intended devices before a release claim. Keep secrets and personal media outside Git.

## Validation

Use behavioral tests for remaining implementation changes, then run `uv run inv pre` followed by `uv run inv test`, including frontend and both browser engines. Speech/content, actual-device and host recovery acceptance stay separate from that software gate. No learning-effectiveness experiment, extra general review loop, push or deployment follows from this plan.
