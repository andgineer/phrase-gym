# P5 — content preparation and integrations

Status: **not started**, 2026-10-11. Depends on P2 contracts and P3 working drills. Some fixture/prompt experiments can start earlier, but generated content must be exercised through the real learning flow before claiming integration complete.

## P5.1 — Vault adapter

- Read explicitly selected phrase notes and attachments into the [import contract](../architecture/data-contracts.md#source-intake).
- Preserve source provenance, language, full examples/dialogues and actor/TTS metadata; prepare stable source-to-target mappings outside the vault.
- Show a dry-run report for edits, duplicates, missing assets and ambiguous matches. Re-import updates content revisions without resetting schedules.
- Reuse or extract useful parsing logic only after tests establish its behaviour. Do not carry over the legacy page's exercise ladder or hint rules.

**Acceptance:** a representative lesson imports with playable audio and linked target turns; repeated import is idempotent; renamed/reordered phrases and two identical surface forms with different meanings are handled correctly. The export command does not change notes, attachments or vault git state. Acquiring/enrolling in additional courses is not part of this adapter.

## P5.2 — Durable preparation pipeline

- Add the leased jobs table/worker, explicit provider configuration, bounded retries, deduplication and structured failure states.
- Define generation outputs for target realizations, chunk boundaries, cue masks, competing-completion analysis, short dialogue turns and on-demand explanations.
- As part of adding a phrase to the app database, have the LLM prepare variants hiding different words/meaningful chunks where the phrase permits useful alternatives. Store checked versions with that phrase for selection across later sessions; do not generate fresh random masks during practice or require a fixed number of variants. Support a single reliable cue and an explicit no-reliable-cue preparation outcome.
- Run deterministic checks before linguistic review: schema, language tags, chunk reconstruction, target/slot references, audio text and allowed help exposure.
- Store provider, model, prompt version, input hash and review outcome. Publish only a reviewed revision; regeneration must not alter an active session.
- Generate model/cue audio once and cache it. Verify the selected language/voice rather than silently substituting an unrelated locale. Preserve supplied actor audio when appropriate.

**Acceptance:** provider timeout/rejection/malformed output leaves a recoverable draft; restart recovers leased work; retries do not publish duplicate targets or assets; practising existing content remains possible while all providers are unavailable. Paid escalation is disabled unless configured deliberately.

Demonstrate the accepted variant lifecycle: add a phrase → LLM prepares candidates → check and store usable variants → practise one → start a later session with another suitable version of the same target. Also cover a phrase with only one reliable variant. All versions must retain one target schedule; reloading an active session must preserve its selected cue.

## P5.3 — Evaluate generated material

Build on P1 fixtures with a small held-out set of unseen expressions. Include idiomatic phrases, reusable frames, word senses, long lines, noninitial distinguishing anchors, plausible synonyms, contractions and cross-language cases relevant to the first release.

Review each candidate for:

- natural and appropriate use in the stated situation;
- fidelity to the selected target meaning and allowed substitutions;
- sufficient identifying material without giving away all required recall content;
- meaningful alternate masks that retain clarity;
- correct full-phrase response, partner line and model/cue audio;
- absence of misleading factual explanations or invented usage rules.

Use real configured providers. Report sample size, accepted/rejected examples, ambiguity reports, preparation latency, retries and observed cost. Show representative failures. Structural pass rate and a model's own rating cannot replace human inspection.

**Acceptance:** no known critical ambiguity or target-meaning error is published from the evaluated pack. Keep human review for initial releases; relaxing it requires separate evidence and a user decision. This evaluation establishes content quality on its sample, not improved long-term learning.

## P5.4 — echo-words handoff

- Begin with manual export/import of the selected structured meaning and examples to prove the contract independently of UI integration.
- Add a durable outbound envelope at selection time with a stable export ID. The current recent-history endpoint is not a reliable import feed after restarts.
- Add an explicit Phrase Gym destination/hand-off action and clear states: queued, received, preparing, ready or failed. Retry using the same ID.
- Preserve the selected sense and source mapping across re-analysis. Use a new adapter boundary; do not force the trainer into Anki's note/card API.
- Carry audio by verified file upload or a tightly scoped source-media transfer. Do not accept unrestricted arbitrary URLs for the server to fetch.

**Acceptance:** selecting a meaning yields one correct target with usable phrases; retry and sender/receiver restarts produce no duplicates; unavailable Phrase Gym does not lose the export; a changed sense is not silently substituted. Acknowledgement means durably received, not necessarily ready to drill.

Switching future output from Anki to Phrase Gym is an explicit rollout decision after the handoff is demonstrated. Do not automatically remove existing Anki content, dual-send all selections, or infer authorization to migrate a collection. When modifying echo-words, re-read its current instructions and run its required checks.

## Optional later work

Speech recognition, targeted automatic feedback and help adapted from recorded difficulties use the same attempt contracts but remain separate experiments. They require a clear recording-consent path, real noisy-speech evaluation and an uncertainty state. No ASR transcript alone is an authoritative pronunciation or recall grade. Free conversation remains excluded.
