# External preparation and reference speech

## Ownership boundary

Phrase Gym defines one versioned prepared-package format and supplies an external-agent guide covering preparation and actual submission. The user's agent understands its source, extracts phrases/optional audio, chooses/calls its text LLM, reviews language quality, handles failures/retries and submits material. Phrase Gym does not parse source formats, mount vaults, run text-generation/review jobs or add a human-approval workflow. Every source uses the same generic intake, including echo-words; Phrase Gym owns no special sender.

The guide covers complete phrases and selected meanings, exact meaningful chunk reconstruction, several [prompts per phrase](decision-targeted-drilling.md#prompts-for-fixed-phrases) checked against the learner's whole phrase set, distinguishing hints, allowed equivalents/frame slots, help in each language's help language only where useful, searchable concepts and complete dialogues. A phrase without an accepted prompt is submitted for study and lookup. Do not invent missing meanings. Semantic review and the learner's acceptance of prompts happen externally; app validation checks structure and bindings.

## Intake and preservation

Stable source/submission identifiers make retries idempotent. Compatible edits preserve targets, schedules, recordings and unchanged chunk evidence; changed meanings require distinct targets. Pinned sessions keep their content. Source ordering and similar wording are not identities.

Optional supplied reference audio needs verified uploaded bytes and exact text/language/role/provenance. Preserve course/actor originals. Reject invalid or supplied-but-untransferred audio rather than treating it as a request for synthesis. Do not fetch arbitrary media URLs. Structural validation and content publication remain atomic internal responsibilities.

## Missing reference speech

Generate genuinely missing whole-phrase/turn audio inside Phrase Gym before practice, cache it and preserve exact speech input, engine/voice, locale/accent, settings and preparation version in its identity. Use one configured primary provider/voice per learning language and a fallback preserving that language and intended locale/accent. Provider failure has bounded fallback/retry and cannot break already prepared practice. Focused durable audio work does not justify a generic text-LLM pipeline or speech grading.

Piper Lessac for US English, Piper Thorsten for German and Edge Sophie for Serbian are starting candidates, not a quality ranking. No macOS-only speech dependency. Serbian speech input may need Cyrillic conversion while displayed text stays unchanged; names, digraph boundaries and foreign words need checking. The Piper serbski_institut voice is Lower Sorbian and must not be selected as Serbian merely from its label. Supplied actor clips always take priority.

Actual speech listening and external-agent semantic review are distinct from schema/container tests. Choose voices using language-specific quality evidence. Private learner recordings are never provider inputs.
