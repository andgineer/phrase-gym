# Material preparation and reference speech

## First input: external phrase batches

Start with bulk English phrases prepared by an external AI from a course the learner already uses. Extraction, source browsing/login and any source-specific tools remain outside Phrase Gym. The app accepts the result; it does not implement a course importer or host a browsing agent. This gives a useful learning set before internal AI generation is needed.

The browser offers **Add material** with a file drop/selection or pasted structured package and an explicit **Import**. Raw JSON is transport for the external agent; the result is a readable list of phrases and their ready/error state. No manual per-card form filling, native Coach window or hidden automatic submission is needed. A file picker is learner-triggered.

The first format is one documented UTF-8 JSON package containing a package ID, source label/link, learning/card/help languages and cards with stable item IDs, complete answer, short explanation, retrieval cues and optional accepted alternatives/detailed explanation. Source links are opaque attribution. Define the exact generated schema and a working external-agent example with implementation; this document does not freeze prototype API fields. Plain unprepared words, URLs alone and slot/dialogue structures are rejected with an explanation of the supported input.

Validate the whole batch before adding it. Tell the learner which item/field needs correction. Identical package-ID/content retries return the same result; changed content under that ID asks for a new package ID. A new package containing the same source/item identity must not silently duplicate or overwrite a learned card: show the conflict and permit explicit replacement as a new card with the old one suspended. No semantic deduplication service is required.

## Reference speech

For the text-only MVP, synthesize English reference audio locally with the existing Piper installation/voice, using its Python API or a bounded invocation. No provider login is needed for speech. Prepare audio once on import and save it; practice plays saved audio without inference. A batch is ready only when its cards and all required reference audio are saved. Show **Preparing audio**, then the confirmed added-card count, or a clear failure with **Retry**. Failed or interrupted imports do not leave half a batch in practice; restarting can offer an explicit retry rather than automatic distributed recovery.

Preserve submitted text and diagnostic reason on failure. Run one import preparation at a time; reject a second attempt with a clear busy message. Ordinary preparation happens outside database transactions; commit the ready batch in one transaction. Do not build a job dashboard, broker, leases, fencing, resumable media-transfer protocol or daily distributed start accounting.

When original course audio support is added, preserve it and identify its provenance; never silently replace it with synthesized speech. Supplied audio must match the answer. Remote audio URLs are not automatically fetched. One acceptable voice for each supported language is sufficient; fallback engines and a voice marketplace are unnecessary.

Audio must be playable and complete, with the intended wording and suitable pronunciation. Technical decoding cannot certify speech quality. Listen to representative output with the learner before calling the voice accepted; automated tests remain silent.

## Later input: echo-words and plain text

After the phrase MVP works, add internal AI preparation for supplied echo-words exports or pasted words with context. Start with a documented export file/paste contract; direct integration with that project's storage or code requires separate authorization. Its source stays read-only.

One bounded request prepares a small batch under the [content rules](decision-targeted-drilling.md#words-after-the-phrase-mvp) and current language choices. Use the learner-selected, already authenticated subscription CLI through one adapter; do not qualify every provider before delivering this path. Existing selection can be reused, with no silent provider switch or paid API fallback. Internal AI means preparation coordinated by this app, not a requirement to run local LLM weights.

Show **Preparing**, **Ready** or an actionable error. Missing sense/context requires a short clarification; a malformed model response never becomes a card. Keep submitted material for explicit retry. Reuse the same validation, audio and import path as external batches. Preparation never reads recordings or changes language preferences.

Before unattended generation is accepted, inspect representative actual outputs for meaning, natural prompts, answer leakage, accepted variants and speech. Use a fresh semantic reviewer for model measurements; schema tests alone are insufficient. This is bounded content qualification, not an open-ended research or review programme.

## Repair and scope

The MVP fixes a bad card by suspension and corrected import. An internal **Recreate** action can follow the echo-words path later if useful; it is not a prerequisite for studying a course batch. The [plan](plan/implementation-plan.md) owns ordering and acceptance. Course extraction plugins, source mutation, native desktop UI and external sender APIs are not hidden prerequisites.
