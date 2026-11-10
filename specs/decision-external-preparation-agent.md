# Material preparation and reference speech

## First input: externally prepared phrases

Start with bulk English phrases prepared by an external AI from a course the learner already uses. Extraction, source browsing/login and any source-specific tools remain outside Phrase Gym. The app accepts the result; it does not implement a course importer or host a browsing agent. This gives a useful learning set before internal AI generation is needed.

The learner never handles packages or JSON: material comes either from an external agent or from the [Coach field](#coach-field-typed-phrases-and-words). An external agent prepares both text and complete reference audio and submits them together through the Phrase Gym API for immediate publication. Its instructions describe material preparation, review and submission; Coach execution details remain in this spec. The browser has no package import, file picker or paste dialog. No manual per-phrase form filling, Coach window or hidden automatic submission is needed.

The external agent's submission is one documented UTF-8 JSON structure containing an input ID, source label/link, learning/card/help languages and phrases with stable item IDs, `text` for the complete wording, short explanation, retrieval cues and optional accepted alternatives/detailed explanation. Phrases also identify their lesson and carry recall hints and usage contexts. Each phrase belongs to one source-scoped lesson with a stable identity. One submission may contain several lessons. `Phrase` is the complete typed record in `phrases`; `phrase.text` is its wording string. Prepared support, reference audio and scheduling fields belong to that same phrase. A Card is only its visual client component, not another data entity. Source links are opaque attribution. Define the exact generated schema and a working external-agent example with implementation; this document does not freeze prototype API fields. Plain unprepared words, URLs alone and slot/dialogue structures are rejected with an explanation of the supported input.

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

Validate the whole submission before adding it. The preparation agent corrects validation errors and retries; unresolved failures retain the concrete error through the existing failure path. Identical input-ID/content retries return the same result; changed content under that ID asks for a new input ID. A new submission containing the same source/item identity must not silently duplicate or overwrite a learned phrase: show the conflict and permit a new phrase after the learner confirms deletion of the old one. No semantic deduplication service is required.

## Reference speech

For material Coach prepares, it synthesizes English reference audio with the existing local Piper installation/voice. No provider login is needed for speech. Use the [durable Phrase Gym handoff](architecture.md#responsibilities) so submission works while Coach is offline. Prepare audio once and save it; practice plays saved audio without inference. A batch is ready only when its phrases and all required reference audio are saved. Submission is saved at once whether or not Coach is running. Preparation is an offline process: the learner submits material when they choose and does not need its result the same day, so the browser never polls preparation and shows no progress; new phrases appear in the library once published. A failed preparation is printed by Coach and becomes a dismissible notice below the learner's cumulative work. The notice arrives with the practice facts the entry already loads (first load, after practice, on return to the page) and is never requested on its own. Dismissal removes the notice in Phrase Gym, so it stays dismissed on every device, and leaves the recorded result unchanged. Submitting the same material again requeues a failed preparation and removes its notice. An external agent can read a submission's result by its ID at any later time. Failed or interrupted imports do not leave half a batch in practice. The external-agent path requires complete validated references with the batch and publishes immediately. The external agent reuses supplied source recordings or chooses an available authorized TTS tool and natural voice for missing audio. The built-in Coach uses Piper; this does not constrain external agents to Piper.

Preserve submitted text and diagnostic reason on failure. Bound concurrent preparation to personal-app scale. Coach performs expensive work outside Phrase Gym transactions; Phrase Gym validates the returned result and commits the ready batch in one transaction. Retry uses stable identities so a repeated submission or result does not duplicate phrases or references. Do not build a job dashboard, broker or general distributed recovery framework.

External agents trust the supplied source's phrase-to-recording association and preserve attribution.
No listening, transcription, ASR or speech-quality analysis is required for routine preparation.
Do not treat their absence as an import acceptance gap. Preserve supplied recordings without
trimming, denoising or other content changes unless the learner explicitly requests an edit;
work on copies and retain the originals. Convert only when necessary for the current mono 16-bit
PCM WAV contract, preserving duration and an accepted sample rate. A compliant file is reused
unchanged. Technical format/size checks remain necessary; they are not semantic audio analysis.
Remote audio URLs are not automatically fetched by Phrase Gym. External-agent TTS selection
needs no application voice marketplace or general provider framework.

For missing audio, the external agent aims for natural speech of the complete source example
using its chosen authorized tool and voice. Routine imports do not require a separate audio-analysis
stage. Technical decoding cannot certify speech quality, and choosing a voice does not establish
learner preference. Attended listening can be requested separately; automated tests remain silent.

## Coach field: typed phrases and words

The header's top right holds a text field and a **Coach** button; it is the learner's only in-app way to add material. The learner types one phrase or word, optionally followed by the sentence it came from, and presses **Coach**. Phrase Gym saves the text to Coach's inbox at once, whether or not Coach is running, then clears the field and briefly confirms; the button is unavailable while the field is empty. Practice in progress continues undisturbed. On a phone the field may take its own header row, but Daily and Extra stay on the first screen.

Coach prepares a phrase as a chunk card under the [phrase rules](decision-targeted-drilling.md#phrases) and a single word as a word-sense card under the [word rules](decision-targeted-drilling.md#words-after-the-phrase-mvp). Prepared items join one standing lesson, **Added with Coach**, kept apart from course lessons so source attribution stays honest. Words from echo-words arrive the same way, typed into the field or inside an external agent's package; there is no export-file import. Direct integration with that project's storage or code requires separate authorization. Its source stays read-only.

One bounded Coach request prepares the item under the content rules and current language choices. Use the learner-selected, already authenticated subscription CLI through one Coach adapter; do not qualify every provider before delivering this path. Existing selection can be reused, with no silent provider switch or paid API fallback. Internal AI means preparation submitted through Phrase Gym and executed by Coach, not a requirement to run local LLM weights.

Like audio preparation, it runs offline: the browser does not track it, and a failure surfaces as the same dismissible notice. Use the supplied context and agent judgment to choose the sense. If the input is insufficient to prepare a coherent card, fail the batch with the concrete reason; do not open a clarification conversation. A malformed model response never becomes a phrase. Keep submitted material so that submitting it again retries preparation. Reuse the same validation, audio and import path as external batches. Preparation never reads recordings or changes language preferences.

### Coach prompts and evaluation

When adding AI preparation to Coach, reuse the content rules from the
[external-agent guide](../docs/src/en/agent-add-phrases.md#content-preparation-rules).
Use separate CLI sessions to prepare material and review it. Assess the review findings,
apply useful corrections and check the changes before publication. Coach is a headless worker:
it takes inbox material, prepares it and publishes through the API. Neither Coach nor the external
preparation agent requests learner opinions or produces preparation reports. Successful API
publication completes the task; unresolved failures use the existing error path. Keep development
and orchestration details in specs, not in external-agent instructions.

Test prompt changes on separate development and held-out sets: tune on the development set,
then check the frozen prompts on untouched test items. If a test finding informs a prompt
change, move that item into the development set and use fresh held-out items next time.
Keep actual outputs, reviewer findings and human judgments distinct.

## Repair and scope

A bad phrase is fixed by confirmed deletion and a corrected submission through Coach or the external agent. An internal **Recreate** action can follow the Coach field path later if useful; it is not a prerequisite for studying a course batch. The [plan](plan/implementation-plan.md) owns ordering and acceptance. The generic Phrase Gym request API is the entry point; course extraction plugins, direct source integration, source mutation and native desktop UI are not hidden prerequisites.
