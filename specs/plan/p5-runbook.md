# P5 — external agent and generic intake

Updated **2026-10-14**. Follow the [accepted external-agent decision](../decision-external-preparation-agent.md) and [remaining implementation plan](03-content-and-integrations.md).

## Intended use

1. Obtain Phrase Gym's preparation prompt, input schema and submission instructions.
2. Give that guide and the chosen source to your own LLM agent.
3. The agent extracts phrases/optional audio, prepares translations, useful explanations and checked gap cues, and submits the prepared material.
4. Phrase Gym validates and stores the package, preserves supplied audio, generates missing reference speech and trains the learner.

Use the same guide for vaults, echo-words and any other source. Only the external agent knows that source format. Preparation/provider failures and delivery retries are external; Phrase Gym does not run an LLM preparation or review process.

Phrase Gym speech configuration selects one primary provider/voice for each learning language plus an appropriate same-language/locale/accent fallback. Speech is generated before practice and cached; existing prepared drills survive provider failure.

## Current implementation gap

The committed runtime still exposes the earlier `export-vault`, `export-existing`, `prepare` and special handoff commands/APIs. Their implementation and old command examples are retained in Git and [historical evidence](evidence/p5-implementation-2026-10-14.md), not the accepted future workflow.

The downloadable source-independent guide/schema and simplified intake/audio path must be implemented, and the obsolete source/LLM orchestration removed. Do not invent new command or endpoint names before they exist; generated OpenAPI/client types remain the authority for actual signatures.

Preserve `.data/vault-test`, all schedules/recordings and historical migrations. External sources remain read-only. No push, deployment, Anki switch or extra fresh review is authorized by this clarification.
