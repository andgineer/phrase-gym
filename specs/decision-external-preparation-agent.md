# Material preparation and reference speech

## First input: externally prepared phrases

Start with bulk English phrases prepared by an external AI from a course the learner already uses. Extraction, source browsing/login and any source-specific tools remain outside Phrase Gym. The app accepts the result; it does not implement a course importer or host a browsing agent. This gives a useful learning set before internal AI generation is needed.

The browser offers **Add material** with a file drop/selection or pasted structured phrase data and an explicit **Import**. An external agent prepares both text and complete reference audio and submits them together through the Phrase Gym API for immediate publication. Its instructions describe this complete workflow without internal preparation details. The result is a readable list of phrases and their pending/ready/error state. No manual per-phrase form filling, Coach window or hidden automatic submission is needed. A file picker is learner-triggered.

The input is one documented UTF-8 JSON structure containing an input ID, source label/link, learning/card/help languages and phrases with stable item IDs, `text` for the complete wording, short explanation, retrieval cues and optional accepted alternatives/detailed explanation. Phrases also identify their lesson and carry recall hints and usage contexts. Each phrase belongs to one source-scoped lesson with a stable identity. One submission may contain several lessons. `Phrase` is the complete typed record in `phrases`; `phrase.text` is its wording string. Prepared support, reference audio and scheduling fields belong to that same phrase. A Card is only its visual client component, not another data entity. Source links are opaque attribution. Define the exact generated schema and a working external-agent example with implementation; this document does not freeze prototype API fields. Plain unprepared words, URLs alone and slot/dialogue structures are rejected with an explanation of the supported input.

External preparation and later Coach prompts follow the [chunk content decision](decision-targeted-drilling.md#phrases)
and the worked preparation rules in the [external-agent guide](../docs/src/en/agent-add-phrases.md#content-preparation-rules).
Cover every supplied source phrase. Select its transferable chunk first, then create natural
gap cues and varied requested wording hints that help retrieve the exact studied expression.
Keep the complete source example in `text` with matching audio;
identify the selected chunk in its explanation. Cue contexts may vary without changing the
chunk or creating extra scheduling identities. Semantic inspection covers chunk boundaries,
completed cue grammar, competing answers and each hint's memory support, separately from schema
acceptance. Requested hints may supply target words or word fragments, but never the entire chunk.
Usage contexts remain short searchable uses; explanations and alternatives remain answer-side help. No new slot structure or per-cue
audio is implied by this preparation format.

Validate the whole submission before adding it. Tell the learner which item/field needs correction. Identical input-ID/content retries return the same result; changed content under that ID asks for a new input ID. A new submission containing the same source/item identity must not silently duplicate or overwrite a learned phrase: show the conflict and permit a new phrase after the learner confirms deletion of the old one. No semantic deduplication service is required.

## Reference speech

For the text-only MVP, Coach synthesizes English reference audio with the existing local Piper installation/voice. No provider login is needed for speech. Use the [durable Phrase Gym handoff](architecture.md#responsibilities) so submission works while Coach is offline. Prepare audio once and save it; practice plays saved audio without inference. A batch is ready only when its phrases and all required reference audio are saved. Show **Pending** or **Preparing audio**, then the confirmed added-phrase count, or a clear failure with **Retry**. Failed or interrupted imports do not leave half a batch in practice. The external-agent path requires complete validated references with the batch and publishes immediately. The agent reuses suitable source recordings or generates missing references before submission.

Preserve submitted text and diagnostic reason on failure. Bound concurrent preparation to personal-app scale and show queued work honestly. Coach performs expensive work outside Phrase Gym transactions; Phrase Gym validates the returned result and commits the ready batch in one transaction. Retry uses stable identities so a repeated submission or result does not duplicate phrases or references. Do not build a job dashboard, broker or general distributed recovery framework.

When an external agent supplies original course audio, preserve it and report its provenance; never silently replace it with synthesized speech. Supplied audio must match the answer. Remote audio URLs are not automatically fetched. One acceptable voice for each supported language is sufficient; fallback engines and a voice marketplace are unnecessary.

Audio must be playable and complete, with the intended wording and suitable pronunciation. Technical decoding cannot certify speech quality. Listen to representative output with the learner before calling the voice accepted; automated tests remain silent.

## Later input: echo-words and plain text

After the phrase MVP works, add internal AI preparation for supplied echo-words exports or pasted words with context. Start with a documented export file/paste contract; direct integration with that project's storage or code requires separate authorization. Its source stays read-only.

One bounded Coach request prepares a small batch under the [content rules](decision-targeted-drilling.md#words-after-the-phrase-mvp) and current language choices. Use the learner-selected, already authenticated subscription CLI through one Coach adapter; do not qualify every provider before delivering this path. Existing selection can be reused, with no silent provider switch or paid API fallback. Internal AI means preparation submitted through Phrase Gym and executed by Coach, not a requirement to run local LLM weights.

Show **Preparing**, **Ready** or an actionable error. Missing sense/context requires a short clarification; a malformed model response never becomes a phrase. Keep submitted material for explicit retry. Reuse the same validation, audio and import path as external batches. Preparation never reads recordings or changes language preferences.

Before unattended generation is accepted, inspect representative actual outputs for meaning, natural prompts, answer leakage, accepted variants and speech. Use a fresh semantic reviewer for model measurements; schema tests alone are insufficient. This is bounded content qualification, not an open-ended research or review programme.

## Repair and scope

The MVP fixes a bad phrase by confirmed deletion and corrected import. An internal **Recreate** action can follow the echo-words path later if useful; it is not a prerequisite for studying a course batch. The [plan](plan/implementation-plan.md) owns ordering and acceptance. The generic Phrase Gym request API is the entry point; course extraction plugins, direct source integration, source mutation and native desktop UI are not hidden prerequisites.
