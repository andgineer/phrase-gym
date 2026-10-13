# Phrase preparation — prompt design and acceptance

Planned implementation work, clarified by the user on **2026-10-13**. This is part of P5 and the full app implementation, not an optional polish task. It expands [P5.2–P5.3](03-content-and-integrations.md) and follows the [targeted-drilling decision](../decision-targeted-drilling.md).

## Current gap

The 44 phrases in the local vault test import each have one manually prepared partial cue. The core can select stored variants using hidden-chunk history, but this pack does not demonstrate different parts being recalled across sessions. Structural checks now reject cues with no visible phrase text; they do not establish that a nonempty hint is good or sufficiently identifying. LLM prompt design, measured cue quality and the intake-to-rotation lifecycle remain unimplemented.

## Required prompt-design deliverables

1. Design and version a preparation prompt that takes the complete source phrase, selected expression/sense or frame, source situation/dialogue context, learning language and native explanation language. Supply relevant neighbouring expressions as competing wording where available. Missing source context must remain explicit.
2. Ask for meaningful chunk boundaries and candidate cues that hide different useful parts of the same phrase while preserving enough identifying wording. The learner always recalls the complete phrase. Do not default to the first words, generate random masks, use first-letter puzzles or treat a broad situation alone as a sufficient cue.
3. Require a competing-completion analysis for each candidate: what other plausible response fits, which visible anchors distinguish the learned expression, what is actually being recalled, and whether the cue supplies a word/frame target that should be hidden. A generic rationale such as “this is a good hint” is insufficient.
4. Request useful, genuinely different variants rather than a fixed count. Reject cosmetic rewordings that hide the same material. Permit one reliable variant for a short or constrained phrase and an explicit no-reliable-cue result when necessary. Record uncovered useful chunks and the reason instead of manufacturing a weak variant to satisfy coverage.
5. Design a separate review prompt/checklist that challenges the candidate using the actual visible cue and plausible alternatives. The generator's confidence or self-approval must not publish a cue. Keep initial human semantic review and a fresh reviewer for real model-result measurements.
6. Define structured draft output, deterministic validation and review outcomes before wiring provider calls into publication. Keep generator explanations and quality-review notes out of the learner's normal card. Save source/input hashes, provider/model, prompt and preparation versions, concrete responses and review outcomes.

Illustrative candidates for **“Not too bad, thanks. And yours?”** are **“_____ thanks. And yours?”** and **“Not too bad, thanks. _____”**. They exercise different parts under the same target and schedule. These are examples for the review set, not a universal template or evidence that every generated variant is adequate.

## Other phrase preparation in the same workflow

The following existing app requirements should share the intake workflow while retaining distinct validation and review responsibilities:

| Preparation | Intended result | Constraint |
|---|---|---|
| Target and meaning analysis | Identify the selected fixed expression, word sense or reusable frame; distinguish required material from allowed substitutions. | Preserve source intent and target identity. A different meaning needs a new objective, not an automatic rewrite of an existing target. |
| Native-language explanation | Concise natural meaning, when to use it, and a useful chunk/construction breakdown where needed. | Explain the actual phrase in context; do not invent grammar rules, force long explanations or display them before the learner requests help. |
| Allowed equivalents and frame slots | Explicitly reviewed contractions, wording alternatives and permissible slot values. | Good English is not automatically equivalent to the selected target. Do not broaden exact-phrase acceptance merely to compensate for an ambiguous cue. |
| Intent and search metadata | Communicative purpose, topic, situation, register and useful related search terms, such as greetings absent from the literal phrase. | Search retrieves existing phrases and lessons. Native-language search and embeddings remain separate choices. |
| Context and partner-turn candidates | Suitable alternate situations or prepared dialogue turns where they improve identification and practice. | Preserve the expression/sense, keep adequate phrase-specific anchors, and introduce no free conversation or compulsory dialogue quota. |
| Source and reference readiness | Flag missing context, uncertain provenance, absent audio and known text/audio mismatches for review. | Preserve source text and original actor/course assets. A text-only LLM result cannot certify what an audio clip says; audio preparation and listening acceptance remain separate work. |

These outputs can be generated in separate bounded stages or a cohesive structured request after representative measurements establish which is more reliable. No new learner-audio analysis or speech-grading feature is implied.

## Quality review and measurements

Before spending provider budget, define representative development fixtures and a separate held-out set, expected failure cases, the measurement plan and available configured providers. Include:

- the current failure case: a broad day-response situation that permits many phrasings;
- short expressions where only one useful variant may exist;
- longer phrases with beginning, middle and ending chunks worth recalling;
- cues needing a noninitial distinguishing anchor and near-duplicate phrases in one lesson;
- fixed phrases, selected word senses and frames, including a visible-target leak;
- contractions, reasonable competing completions, missing context and no-reliable-cue outcomes;
- supported learning languages and native explanations relevant to the release.

Inspect every concrete real-provider result. Record semantic acceptance/rejection separately from automated conformance, with representative rejected examples and reasons, latency, retries and observed cost. A fresh reviewer must inspect the real model-result measurements; mocks and a second model's approval alone cannot establish cue quality. No learner effectiveness experiment, calibration session or reopening of P1 follows from this work.

Review each cue for naturalness, fidelity to the selected meaning, sufficient distinguishing material, hidden useful recall content and honest supplied support. Strengthen or withhold an ambiguous cue rather than grade an arbitrary wording choice as forgetting. Keep phrases without reliable cues available for supported study and mark retrieval preparation incomplete.

## Integration acceptance

- [ ] Prompts, structured output, validation and review criteria are versioned and supported by concrete reviewed provider results.
- [ ] Adding a representative phrase produces multiple accepted stored masks that hide different meaningful parts; one-variant and no-reliable-variant cases also work.
- [ ] The stored variant sets fit the selector's comparable-support rules. Demonstrate that useful variants are actually reachable; do not store alternatives that the current hidden-chunk-count filter can never choose.
- [ ] Later ordinary sessions select different suitable hidden parts of the same target using presentation/recall history. Reload keeps the active session's pinned cue and content revision.
- [ ] All variants, contexts and dialogue appearances retain one target schedule. Prepared support is recorded accurately; partial-cue recall is not relabelled independent speech.
- [ ] Generated native explanations and search metadata are checked through Show & explain and representative concept queries against the real app.
- [ ] Already imported phrases, including the current 44-phrase local pack, can be prepared or regenerated without duplicate targets, reset schedules, lost recordings or mutation of earlier session manifests.
- [ ] Timeout, malformed output, failed semantic review and provider outage leave recoverable drafts. Published practice uses stored material and never waits for an LLM between cards.

P5 is not complete merely because JSON validates, a cue has some visible text, or the core has a rotation algorithm. It must demonstrate preparation, review, publication and useful later cue selection together.
