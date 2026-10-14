# Remaining implementation work

This plan contains unfinished work only. Remove entries as they are completed; lasting constraints belong in the linked decisions, and code/tests describe delivered behavior.

## External-agent intake and reference speech

Follow the [external-agent boundary](../decision-external-preparation-agent.md). The runtime still contains source parsing, application text-LLM preparation/review and special handoff workflows that do not belong in the app.

- Remove vault/legacy-note parsers, text-provider configuration/calls, generic LLM jobs/review APIs and source-specific handoff interfaces. Preserve data, migrations, generic validation/publication, immutable revisions, chunk reconciliation, media storage and reusable speech adapters. Remove tests for discarded responsibilities and refresh generated API contracts.
- Deliver a source-independent preparation/submission guide and the actual versioned schema. Include stable identities, meaning/chunk/cue/help/dialogue rules, optional-media upload/binding, validation feedback, idempotent retries and compatible updates. Document real commands/endpoints once implemented.
- Provide one generic prepared-material intake path without application linguistic review or operator approval. Verify unrelated sources, identical-text/different-meaning targets, retries, compatible updates and supplied-audio preservation. Demonstrate prepared cues through actual selection and pinned reload without resetting progress.
- Isolate missing-reference speech into focused bounded audio work with per-language primary/fallback and complete-phrase caching. Verify failure/retry, partial resume and cache invalidation without replacing good supplied assets. Language-specific voice selection/listening remains open.
- Assess the guide using representative externally prepared material and held-out examples: short/long phrases, idioms, frames, contractions, ambiguous completions, missing context and relevant language/native-help combinations. Demonstrate several useful cues, one reliable cue and supported study without a retrieval cue. Inspect semantic quality separately from structural conformance; provider measurements need a planned fixture set and a fresh reviewer.

External sources remain read-only unless separately authorized. Preparation/provider choice and delivery retries belong to the external agent.

## Device acceptance

Complete the [device gaps](../open-problems.md) on actual iPhone Safari over trusted HTTPS, with speaker and headphones. Check longer/successive takes, saved replay, interruption/reopening, temporary server failure/retry and accessibility. Record only actionable gaps and current acceptance limits, without accumulating review logs in specs.

## Private release and recovery

- Measure the built app and bounded speech work on the proposed Oracle VM or always-on laptop: idle/API/media peaks, disk growth and impact on neighboring services. Do not assume speech models fit a shared 1 GB host.
- Prepare a concrete host/access/backup proposal and operational documentation covering configuration, start/stop/update, health checks, storage/logging and recovery. Host choice and deployment require an explicit user request.
- Demonstrate a consistent database/media backup and isolated restore, including every language, settings, schedules/history, every learner take, reference provenance and unfinished audio work. Rehearse upgrade and rollback using matching snapshots where needed.
- Verify actual private access and health from intended devices before a release claim. Keep secrets and personal media outside Git.

## Validation

Use behavioral tests for remaining implementation changes, then run `uv run inv pre` followed by `uv run inv test`, including frontend and both browser engines. Speech/content, actual-device and host recovery acceptance stay separate from that software gate. No learning-effectiveness experiment, extra general review loop, push or deployment follows from this plan.
