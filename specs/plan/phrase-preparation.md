# Phrase preparation — instructions for the external agent

Updated **2026-10-14** under the accepted [external-agent decision](../decision-external-preparation-agent.md). Phrase Gym supplies the prompt, versioned input schema and submission instructions. The user's agent runs preparation outside Phrase Gym and understands the user's source format.

The current runtime still contains the earlier application-owned LLM workflow. The following is the accepted replacement, not a claim that the runtime or a downloadable agent guide has already been simplified.

## Prompt contents

1. Read the selected complete phrase, expression/sense/frame, language, native explanation language and available context. Keep missing context explicit; do not invent a meaning. Use stable identifiers rather than a source's current ordering.
2. Preserve the complete phrase and meaningful chunk boundaries. Every cue requests the whole phrase; chunks reconstruct it exactly.
3. Prepare useful different hidden parts while keeping identifying visible wording. Avoid random gaps, first-letter puzzles, punctuation-only support and situation-only prompts. One reliable cue is enough; zero reliable retrieval cues is an honest result.
4. Challenge each visible cue with realistic competing completions. Check identifying anchors, hidden required material, target leaks and reviewed equivalent/slot boundaries. Do this externally before submission, without exposing quality rationales as learner instructions.
5. Supply a natural native-language translation/meaning. Add concise usage, nuance or construction explanation only when useful; do not require filler or another language's explanation.
6. Prepare intent/topic/situation/synonym terms that retrieve the actual stored target. Preserve complete dialogues and explicit turn meanings when included.
7. Preserve optional complete phrase/turn recordings and their source/performer or generation metadata. Upload them through the documented intake path; never substitute a headword clip or silently drop a supplied actor clip.
8. Submit the prepared package using the actual schema/API. Explain stable IDs, idempotent delivery, media binding, validation errors and compatible updates without resetting schedules or earlier sessions.

The guide must be usable by the user's chosen LLM agent with any unique source. No Phrase Gym vault parser, echo-words adapter, text-provider credentials, generation worker or human-approval API is required.

## Quality and demonstration

Use public development examples and separate held-out examples to assess the prompt through an external agent. Cover short/long phrases, idioms, word senses, frames, noninitial anchors, realistic alternatives, contractions, missing context and relevant language/native-help combinations. Inspect actual prepared material separately from schema checks.

Demonstrate multiple useful cues, a single reliable cue and supported study without a retrieval cue. Submit prepared material through generic intake, practise it, then confirm later selection and pinned reload. Preserve one target schedule and honest support evidence.

External preparation failures, provider choice, retries and semantic review belong to the agent/user. Phrase Gym returns structural/storage feedback and never calls a text LLM to repair the submission. Existing measured results remain [historical evidence](evidence/p5-semantic-review.md) of the previous prompt/workflow.

Actual reference-speech quality remains a separate audio check. No learner effectiveness experiment or reopening of P1 follows from this work.
