# Material preparation and reference speech

## First input: externally prepared phrases

Start with bulk English phrases prepared by an external AI from a course the learner already uses. Extraction, source browsing/login and any source-specific tools remain outside Phrase Gym. The app accepts the result; it does not implement a course importer or host a browsing agent. This gives a useful learning set before internal AI generation is needed.

The learner never handles packages or JSON: material comes either from an external agent or from the [Coach page](#coach-page-typed-phrases-and-words). An external agent prepares both text and complete reference audio and submits them together through the Phrase Gym API for immediate publication. Its instructions describe material preparation, review and submission; Coach execution details remain in this spec. The browser has no package import, file picker or paste dialog. No manual per-phrase form filling, Coach window or hidden automatic submission is needed.

The external agent's submission is one documented UTF-8 JSON structure containing an input ID, source label/link, learning and meaning-layer languages and phrases with stable item IDs, `text` for the complete wording, short explanation, retrieval cues and optional detailed explanation. Phrases also identify their lesson and carry recall hints and usage contexts. Each phrase belongs to one source-scoped lesson with a stable identity. One submission may contain several lessons. `Phrase` is the complete typed record in `phrases`; `phrase.text` is its wording string. Prepared support, reference audio and scheduling fields belong to that same phrase. A Card is only its visual client component, not another data entity. Source links are opaque attribution. Define the exact generated schema and a working external-agent example with implementation; this document does not freeze prototype API fields. Plain unprepared words, URLs alone and slot/dialogue structures are rejected with an explanation of the supported input.

External batches and Coach cards with authored examples follow the [chunk content decision](decision-targeted-drilling.md#phrases)
and the worked preparation rules in the [external-agent guide](../docs/src/en/agent-add-phrases.md#content-preparation-rules).
Cover every supplied source phrase. Select its transferable chunk first, then create natural
gap cues and varied requested wording hints that help retrieve the exact studied expression.
Keep the complete source example in `text` with matching audio;
identify the selected chunk in its explanation. Cue contexts may vary without changing the
chunk or creating extra scheduling identities. Semantic inspection covers chunk boundaries,
completed cue grammar, competing answers and each hint's memory support, separately from schema
acceptance. Requested hints may supply target words or word fragments, but never the entire chunk.
Usage contexts remain short searchable uses; explanations remain answer-side help. No new slot structure or per-cue
audio is implied by this preparation format.

Validate the whole submission before adding it. The preparation agent corrects validation errors and retries; unresolved failures retain the concrete error through the existing failure path. Identical input-ID/content retries return the same result; changed content under that ID asks for a new input ID. A new submission containing the same source/item identity must not silently duplicate or overwrite a learned phrase: show the conflict and permit a new phrase after the learner confirms deletion of the old one. No semantic deduplication service is required.

## Reference speech

Coach synthesizes the complete revealed wording once, using a local Piper voice for the card's learning language; each learning language has its own voice. A language without a configured voice is a retryable service failure reported before any model call, never a language rejection, and the other languages keep working. Practice plays the saved reference. Phrase Gym accepts text while Coach is offline and publishes a batch only after validation and receipt of all required audio. Bound concurrent preparation to personal-app scale and perform expensive work outside the transaction. Stable identities prevent duplicate phrases or references after interruption. See the [handoff](architecture.md#responsibilities).

An external agent supplies validated audio with its batch for immediate publication and can read the result by submission ID. It reuses supplied recordings or chooses an available authorized TTS tool and natural voice for missing audio.

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

## Coach page: typed phrases and words

The learner pastes a word, phrase or source sentence into **Phrase or word** and presses **Send** or Enter. A word or phrase can include an optional source sentence. The four-row box disables the full-width **Send** button while empty. Pasted line breaks become spaces for submission. Phrase Gym saves the request, clears the box and confirms it. A note explains that published cards appear in Library under **Added with Coach**, separate from course lessons. Opening Coach retains the open card's media as usual. Returning to Practice and opening Daily resumes its server view.

A Library search with no match says **Not in your Library yet.** and offers **Send “…” to Coach**. Matching results end with **Not it? Send “…” to Coach**. Both submit the trimmed search through the same route and show confirmation or an error. An empty library links to Coach. Words from echo-words can be submitted this way or in an external agent's package. Its storage and code stay read-only.

### Input decisions

Submissions from both Coach and Library capture the learning language of the page in use when the learner presses Send. Coach judges the text in that language. The meaning layer follows [the learner's translation choice](decision-personal-preferences.md), and detailed help uses the native language. English and German apply it today; apply the same decision in each further language.

| Submitted text | Coach outcome |
|---|---|
| Valid word or expression, including a regional spelling or inflected form | Preserve the wording. Use the source to identify the intended expression and any sense it establishes. A valid word alone can be practised for its form. |
| Obvious typo with only one plausible correction in the available context | Use the corrected form automatically. Keep the original and show the learner the change, for example **definately → definitely**, in the Coach result. |
| Wrong language, unintelligible text or several plausible corrections | Reject the input and keep it out of practice. A plausible guess is insufficient to change what the learner studies. |

A valid form with several meanings can support wording practice. Claim a particular meaning only when the available evidence supports it. A valid variant needs no correction to a preferred dialect. Coach never asks the learner to select a meaning, approve a correction or review a card before publication.

### Results in Coach

The **Coach** inbox shows rejected submissions with the original text, selected learning language, concrete reason and **Dismiss**. The learner may edit and submit new material if they choose. Repeating unchanged rejected input returns its existing result. Dismissal persists across devices and never requeues work or deletes its recorded result. Temporary login, quota, timeout, audio or other service failures have a distinct result with a reason and a retry when useful. Coach logs these failures, and Practice shows a dismissible notice below cumulative work with the practice facts already loaded. Resubmitting after a service failure retries it and clears that notice. An outage is insufficient evidence for a language rejection. The browser reads inbox results when Coach opens or refreshes, without progress polling or a live job dashboard.

The learning language, submitted wording and optional source sentence identify repeat material. Ignore repeated spaces and inconsequential casing when comparing, but preserve distinctions such as German **Morgen** and **morgen**. Keep the submitted text. A duplicate reports **Already waiting for Coach** or **Already in your Library** as appropriate. A new submission, an explicit retry after a service failure, or a new submission after deletion of its card reports **Saved for Coach**. A different source or learning language can identify a different item.

### Coach prompts and evaluation

Coach automatically prepares the safest useful card permitted by the [content decision](decision-targeted-drilling.md#basic-wording-practice). A word-sense card remains an ordinary phrase record. Coach prompts and the external-agent guide use the same files of authoring rules for enriched content: language-neutral rules that speak of the learning, meaning-layer and native languages, plus a section for each language that needs its own notes. German notes cover nouns practised with their article, capitalisation as part of the word (**essen** and **das Essen** are both valid and never corrected into each other, while a lowercase noun after an article is an obvious typo) and separable verbs, whose cue frame keeps the verb whole because a gap covers one contiguous span. Keep language judgments in vendor-neutral prompts and mechanical contract checks in code. Adapters own structured result delivery and bounded execution under the [provider and CLI policy](architecture.md#coach-provider-selection). A malformed response cannot authorize publication.

### Preparation by provider

Card quality on the paid routes comes from strong models. Weaker, cheaper models cannot produce quality cards, so token savings must come from how Coach calls the strong models, never from switching models.

| Route | Preparation |
|---|---|
| OpenAI subscription | `gpt-6.1-sol` at medium effort prepares the card. Separate calls on the same model review it and revise it, for at most two review and revision rounds. |
| Anthropic subscription | `claude-opus-5-5` at medium effort, in the same prepare, review and revise flow. |
| Gemini free | One recognition call with `gemini-3.8-flash-low` under the Antigravity routing policy. The program builds [basic wording practice](decision-targeted-drilling.md#basic-wording-practice) from the confirmed text. |
| Gemini subscription | Preparation design remains open until measured on that subscription. |

The paid routes keep this flow until a token-saving change passes measurement on the same held-out inputs with fresh blind semantic review: no fewer complete cards, no lower cue quality and no materially wrong result. A change that lowers card quality is rejected whatever it saves. Most of each paid call is CLI overhead and extra rounds spent fetching the task, not Coach content, which is where savings are sought first.

On subscription routes the review checks the original submission, its learning language, target, correction, source sentence and every addition against the shared content rules, and the revision corrects what it finds. A card that still fails the mechanical checks after the last revision is not published. An unavailable or interrupted review or revision is a service failure, and its incomplete draft is discarded. Every published exercise requires validation, matching audio and atomic publication.

Gemini free accepts lower card quality: a gap in the learner's own sentence or a partial form of the wording, without meaning cues or examples. Only language correctness is required there: keep valid input, correct only an obvious typo and reject wrong-language or unintelligible text. In the text-only proof of concept the free-route verifier accepted both a misleading synonym and a gloss that reversed the action in its sentence; the wording-only exercise avoids those errors.

**Evidence.** Branch `coach-plan-trial` on origin implements the cheaper alternative: `claude-haiku-5-5` writes and `claude-sonnet-5` checks, or `gpt-5.6-luna` writes and `gpt-5.6-terra` checks, all at low effort. On 12 held-out English inputs per route, judged blind by a fresh reviewer on 2026-11-25, the strong flow produced 10 full cards out of 12 on each route, with average cue quality 4.8 on Claude and 4.2 on Codex and no material error. The cheaper pairs produced 2 and 0 full cards, average cue quality 2.6 on both, and the Claude pair failed to correct **thier** to **their**. The weaker writers under-delivered: luna returned no examples, and haiku often returned too few hints for one justified checker rejection to leave a full card. Most of their cards fell back to letter skeletons such as **co…** for **colour**, which the reviewer judged unusable. They also missed the token goal, using 89.5% of the strong flow's tokens on Claude and 80% on Codex. Claude's list price fell to about 26%, but only because the models are cheaper, which does not justify the quality loss.

## Repair and scope

A bad phrase is fixed by confirmed deletion and a corrected submission through Coach or the external agent. An internal **Recreate** action can follow the Coach page path later if useful; it is not a prerequisite for studying a course batch. The [plan](plan/implementation-plan.md) owns ordering and acceptance. The generic Phrase Gym request API is the entry point; course extraction plugins, direct source integration, source mutation and native desktop UI are not hidden prerequisites.
