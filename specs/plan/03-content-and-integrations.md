# P5 — external-agent intake and reference speech

Updated **2026-10-14**. The accepted [external-agent decision](../decision-external-preparation-agent.md) supersedes application-owned source adapters, LLM preparation/review and special echo-words handoff. The existing P5 implementation still contains that superseded scope; simplification is remaining work. Earlier measurements and the three-round review remain historical evidence.

## P5.1 — Preparation prompt and input rules

Phrase Gym provides a source-independent prompt for the user's external LLM agent, together with the actual versioned input schema and submission instructions.

The prompt tells the agent how to prepare:

- complete phrases and selected meanings, with stable identities;
- natural native-language translations and useful explanations without filler;
- meaningful chunks and checked gap cues hiding different useful material;
- competing completions, allowed equivalents and frame slots;
- concept search terms and complete dialogues where supplied or useful;
- optional exact phrase/turn audio with its provenance.

The external agent reads its own source, chooses/calls its LLM, reviews the result, handles preparation failures and submits prepared material. Phrase Gym does not know the source format or run this process. Do not manufacture a cue when none is reliable or invent missing source meaning.

**Acceptance:** an agent can obtain the prompt/schema, prepare material from an arbitrary source and submit it without installing a Phrase Gym source adapter or configuring a text LLM inside the app. The guide includes both preparation and actual delivery. Prompt quality is assessed using representative external-agent results; linguistic review is external.

## P5.2 — Generic prepared-material intake

Use one prepared-package format and one generic intake path. Keep structural validation and durable storage inside Phrase Gym; remove in-app LLM review, operator approval objects and preparation orchestration.

- Check language tags, exact chunk reconstruction, cue/target/slot references and complete dialogue bindings.
- Accept translations, necessary explanations and checked cues as prepared by the external agent. The app does not certify semantic quality or call an LLM to repair input.
- Give clear receipt/errors; identical retries must not duplicate targets, media or schedules.
- Compatible edits preserve target identity, learner progress/recordings and unchanged chunk evidence. Changed meanings use distinct targets. Pinned sessions retain their earlier content.
- Accept optional uploaded reference audio with exact text/language/role/provenance. Preserve supplied originals. Reject invalid or supplied-but-untransferred media instead of silently replacing it.
- Keep arbitrary URL fetching, vault filesystem access and source-specific protocols outside the app.

**Acceptance:** the same format works for unrelated sources, including an external agent using echo-words. Different meanings with identical surface text remain separate; retries/updates preserve progress. Prepared masks work through actual session selection, and reload preserves the chosen cue.

## P5.3 — Missing reference speech

Keep one configured primary provider/voice per learning language, selected using actual language-specific speech quality. Existing Piper/Edge findings are a starting point, not a declaration of the best voice.

Keep matching-language/locale/accent fallback. Generate complete phrases and dialogue turns once, cache them with exact speech text, voice, settings and version, and preserve Serbian display text while preparing speech input. Supplied course/actor audio takes precedence.

Speech failures have bounded fallback/retry, retain recoverable audio work and cannot break existing prepared drills. This responsibility may use focused durable audio jobs; it does not retain text-generation or linguistic-review jobs. Do not add learner speech grading.

**Acceptance:** supplied clips remain unchanged; missing clips are playable and correctly bound. Provider/network failure reaches the intended fallback or explicit audio failure. Cache/version changes and partial resumes do not reuse stale generated clips. Voice-quality selection/listening is separate from software conformance.

## P5.4 — Remove superseded scope

Remove the vault/legacy-note parsers, application text-provider calls/configuration, LLM preparation worker/review APIs and dedicated echo-words handoff interfaces. Preserve reusable generic intake, content validation/publication, immutable revisions, chunk reconciliation, media storage and speech fallback.

Keep historical migrations/evidence and existing learner data. Remove obsolete tests together with their removed responsibilities; retain meaningful coverage for remaining behavior. Refresh generated API types and run the ordered project gate after implementation.

There is no Phrase Gym-owned echo-words sender to defer. An agent on that side uses the same prompt and generic intake. No external repository changes, Anki migration, push or deployment are authorized here.

## Review scope

The earlier [three-round review](evidence/p5-review-loop-2026-10-14.md) concerns the superseded implementation. Do not continue its source-adapter acceptance matrix as the new plan. This clarification alone does not authorize another fresh review session.
