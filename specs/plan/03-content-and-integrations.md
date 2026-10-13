# P5 — content preparation and integrations

Status: **not started**, updated 2026-10-13. P2 contracts and P3 working drills are available. The local 44-phrase test pack has one manual partial cue per phrase; LLM preparation and different useful masks are still outstanding. Generated content must be exercised through the real learning flow before claiming integration complete.

This phase completes the preparation/intake coverage in the [consolidated plan](implementation-plan.md). Manual content in P2–P3 is a staged implementation path, not a deferral of the accepted add-time cue preparation beyond the full planned release.

## P5.1 — Vault adapter

- Read explicitly selected phrase notes and attachments into the [import contract](../architecture/data-contracts.md#source-intake).
- Preserve source provenance, language, full examples/dialogues and actor/TTS metadata; prepare stable source-to-target mappings outside the vault.
- Show a dry-run report for edits, duplicates, missing assets and ambiguous matches. Re-import updates content revisions without resetting schedules.
- Reuse or extract useful parsing logic only after tests establish its behaviour. Do not carry over the legacy page's exercise ladder or hint rules.

**Acceptance:** a representative lesson imports with playable audio and linked target turns; repeated import is idempotent; renamed/reordered phrases and two identical surface forms with different meanings are handled correctly. The export command does not change notes, attachments or vault git state. Acquiring/enrolling in additional courses is not part of this adapter.

## P5.2 — Durable preparation pipeline

- Implement the [phrase-preparation prompt design and acceptance plan](phrase-preparation.md): versioned generator and review prompts, competing-completion analysis, useful different hidden chunks, representative real-provider measurements and preparation of related meaning/explanation/search material. This is required app implementation work, not satisfied by manually adding one cue per phrase.
- Add the leased jobs table/worker, explicit provider configuration, bounded retries, deduplication and structured failure states.
- Define generation outputs for target realizations, chunk boundaries, cue masks, competing-completion analysis, short dialogue turns and on-demand explanations.
- As part of adding a phrase to the app database, have the LLM prepare variants hiding different words/meaningful chunks where the phrase permits useful alternatives. Store checked versions with that phrase for selection across later sessions; do not generate fresh random masks during practice or require a fixed number of variants. Support a single reliable cue and an explicit no-reliable-cue preparation outcome.
- Under the current learning-design recommendation, identify the required hidden material and supplied support for each cue so selection can target missing/weak coverage. Prepare suitable alternate situations/partner turns where useful, preserving the fixed expression or selected word/frame meaning, adequate disambiguation and allowed alternatives. These are ordinary-practice material, not separate transfer tests; no fixed context quota is required.
- Prepare concise native-language meaning/use explanations with an optional chunk/construction breakdown. Make them usable for repairing misunderstanding after repeated difficulty; avoid adding long generic instruction text to the drill.
- Under the recommended search design, prepare/version intent, situation, topic and likely synonym metadata with the stored phrase, then update the language-scoped search index. Cover concept queries absent from literal wording, not just phrase substrings. Search returns existing targets/lessons; no generated replacement answers. Choose whether more advanced retrieval is needed from representative query checks, not a runtime LLM dependency for opening a phrase.
- Run deterministic checks before linguistic review: schema, language tags, chunk reconstruction, target/slot references, audio text and allowed help exposure.
- Store provider, model, prompt version, input hash and review outcome. Publish only a reviewed revision; regeneration must not alter an active session.
- Generate reference/cue audio once and cache it, preserving supplied actor/course audio first. Follow the inspected [echo-words audio guidance](../reference/echo-words-audio.md): per-language voices, Serbian Cyrillic speech preparation without changing displayed script, whole-phrase synthesis, same-language fallback and explicit retryable failures. Include voice/locale, exact speech text and preparation version in media/job identity. Do not require online synthesis during a drill or borrow echo-words' word-only Commons lookup for complete phrases.

**Acceptance:** provider timeout/rejection/malformed output leaves a recoverable draft; restart recovers leased work; retries do not publish duplicate targets or assets; practising existing content remains possible while all providers are unavailable. Paid escalation is disabled unless configured deliberately.

Demonstrate the accepted variant lifecycle: add a phrase → LLM prepares candidates → check and store usable variants → practise one → start a later session with another suitable version of the same target. Also cover a phrase with only one reliable variant. All versions must retain one target schedule; reloading an active session must preserve its selected cue.

Audio acceptance: preserve supplied course/actor clips; create missing complete phrase/turn clips with the selected language/voice/accent; keep Serbian display text unchanged while preparing appropriate speech input. Use real generated phrase samples for content/listening checks during implementation, alongside software checks for exact request text, playable output, provenance, retry and cache invalidation when voice/text/settings change. Do not use the Lower Sorbian voice identified in the echo-words findings or another language's fallback. Provider outages leave recoverable jobs and cannot break existing prepared drills. These are implementer checks, not learner efficacy experiments.

## P5.3 — Evaluate generated material

Follow the [prompt-quality measurement plan](phrase-preparation.md#quality-review-and-measurements) before spending provider budget: inspect availability, select representative development and held-out fixtures, and have a fresh reviewer inspect concrete model-result measurements. Record semantic review separately from automated conformance.

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
- Reuse transferred audio only when it matches the exact phrase/dialogue turn and intended language/accent. Echo-words can hold submitted-text, headword and context recordings; a headword-only clip cannot serve as the complete phrase reference. Preserve provenance and otherwise prepare the missing clip through the audio adapter above.

**Acceptance:** selecting a meaning yields one correct target with usable phrases; retry and sender/receiver restarts produce no duplicates; unavailable Phrase Gym does not lose the export; a changed sense is not silently substituted. Acknowledgement means durably received, not necessarily ready to drill.

Switching future output from Anki to Phrase Gym is an explicit rollout decision after the handoff is demonstrated. Do not automatically remove existing Anki content, dual-send all selections, or infer authorization to migrate a collection. When modifying echo-words, re-read its current instructions and run its required checks.

## Optional later work

Speech recognition, targeted automatic feedback and help adapted from recorded difficulties use the same attempt contracts but remain separate experiments. They require a clear recording-consent path, real noisy-speech evaluation and an uncertainty state. No ASR transcript alone is an authoritative pronunciation or recall grade. Free conversation remains excluded.
