# Useful language and retrieval cues

Phrase Gym helps an adult bring useful expressions and word senses into their own speech. It complements courses, listening, reading and real communication. Retrieval on a phrase prompt is an intermediate outcome, not evidence that a phrase was used naturally.

The first MVP accepts externally prepared English phrase batches. Freer speaking and adult engagement follow, before words from echo-words and further languages. The [delivery ladder](plan/implementation-plan.md) defines scope; the content rules below define what a prepared phrase contains.

## Phrases

Each card targets a useful reusable chunk, such as **strongly believe**, **totally convinced**,
**Trust me**, or **Here's another great thing about that**. The goal is to make conventional
wording available in new speech. Memorizing incidental content about a particular product is
not the goal. A self-contained routine can be the entire target; a longer course sentence is
usually an example containing a smaller transferable expression.

Cover every supplied source phrase; package size is a batching limit, not a selection quota.
Choose one coherent chunk per card, preserving its lexical core and required grammar. Keep
`text` as the complete unchanged course sentence with matching reference speech and attribution.
The explanation explicitly identifies the target chunk, its meaning/use and the surrounding
example context. This uses the current complete-example contract; it does not require a slot
editor, new target field, generated per-cue answers or automatic speech evaluation.

The default cue is a natural sentence with the whole target chunk replaced by one constant-length
gap and, when useful, a short meaning gloss. The visible context carries the incidental proposition.
For example, **We __________ that this approach will set us apart from competitors. (=We truly
think)** asks for **strongly believe**; **We're __________ that this tool will make data analysis
more efficient. (=We are completely sure)** asks for **totally convinced**. Grammar distinguishes
these two taught chunks without revealing either lexical core; other natural English answers
remain possible. For a complete routine, a brief exchange with the response blank can work.

Prefer two genuinely different contexts that accept the same unchanged chunk; one is acceptable
when a second would be forced. Keep prompts comprehensible without learning incidental vocabulary.
A partner line is optional. Check every completed cue for naturalness, grammar and meaning, then
try likely competing chunks from the whole source. Natural alternatives can remain; a requested
wording hint helps identify the studied expression without displaying its complete answer.
Generated contexts are preparation, not quotations from the course.

The bracketed cue gloss supplies meaning; requested hints help retrieve the exact studied wording
among natural alternatives or restart recall when the learner is stuck. Choose a diagnostic word,
a short fragment of the chunk or part of a word, rather than another definition or abstract label.
For **strongly believe**, **strongly** or **bel…** can activate recall of the rest. Never supply
the entire selected chunk, even if the source sentence contains additional words. Short targets
may need partial-word hints so the learner still has something meaningful to retrieve.

Prepare two to six distinct hints of one or two words/fragments. Vary the word or fragment supplied
so practice does not depend on one fixed hint. Each must work alone with every cue because the
app chooses randomly and avoids an immediate repeat when possible. They are not an ordered ladder.
Review each hint's intended memory nudge and what remains to retrieve. This is supported recall,
not a claim of unaided retrieval or a research-established optimum amount of support.

The learner retrieves the missing chunk and may say the completed cue sentence. The self-rating
concerns the target's form and appropriate use, not exact reproduction of the example's incidental
content. The revealed sentence/audio illustrates the same chunk in its original source context;
it is not necessarily the full answer to a newly authored cue. Supported recall after Hint remains
distinct from the original unaided attempt. An acceptable alternative is not a language error;
revise or replace a misleading card rather than treating ambiguous preparation as forgetting.

Prepared records also carry short searchable `usage_contexts`, useful accepted alternatives and
optional detailed native-language help. The short explanation appears during study and after
reveal; detailed help is available on request. Neither reveals the target before an independent
attempt. A card's scheduling identity is shared across its cues; new contexts do not create
duplicate cards. Daily and Extra use the same cues and hints. Phrase lookup searches the answer,
explanations, cues and usage contexts, shown as **Used for** in details.

The [external-agent guide](../docs/src/en/agent-add-phrases.md#content-preparation-rules) owns the
preparation procedure, worked examples, rejection checks and schema mapping. Review every card's
source coverage, chunk boundaries, cue completions and varied wording hints before publication.
A semantic review is separate from API acceptance; no fixed example establishes quality for an entire course.

**Evidence and limits:** [collocation exercises](research/phrase-prompts.md#one-phrase-per-moment)
support studied constructed responses in the tested conditions; that study used first-letter
support and did not test these hint rules. [Retrieval support](research/phrase-prompts.md#least-help-that-still-succeeds)
motivates preserving a real retrieval demand, not a universal ban on supported practice.
[Formulaic language](research/automatic-use.md) motivates reusable expressions rather than whole
incidental sentences. Contextual chunk gaps and varied partial-wording hints are learner-directed product
choices informed by this evidence; no reviewed study establishes their superiority for this app
or guarantees spontaneous conversational transfer.

## Words after the phrase MVP

Each word-sense entry teaches one intended sense through comprehensible chunks, a short definition and two or three different contexts. A raw echo-words entry needs enough context to choose its sense; otherwise the app asks for a short clarification instead of inventing one. Keep source identity so retrying the same entry does not duplicate it.

For English at the learner's stated B2 level, use a short English context with the target word missing, plus a sense cue if needed; ask the learner to say the complete chunk. For beginner German, use the Russian meaning of a short chunk and ask for the German chunk. Explain article/plural or another form only when relevant. Grade retrieval of the target sense/form, not an unrelated slip elsewhere in the sentence. Accepted alternatives must fit the intended meaning.

Use one productive schedule per word sense. Understanding a definition is not successful retrieval of the spoken form. Contexts and explanations follow [personal preferences](decision-personal-preferences.md); English-only help is a learner choice, not a universal teaching rule.

For troublesome near-synonyms or lookalikes, give a short accurate contrast, including overlap. Avoid introducing a batch of confusables together. No research establishes an exact prerequisite graph or later-day unlocking algorithm, so none is required.

**Evidence:** [word learning](research/words-in-context.md) supports deliberate form/meaning practice, productive retrieval and context variation. A gap is one convenient task format, not a demonstrated bridge to natural conversation. Two or three contexts is a practical default, not an experimentally established optimum.

## Transfer to active use

Spaced spoken retrieval is the MVP's contribution: make useful wording available from meaning, with feedback and varied cues. [Conversational-uptake evidence](research/automatic-use.md) supports instruction combined with meaningful speaking, including increased use in unscripted classroom conversations. It does not guarantee lasting everyday use or automatic speech for this app.

The [practice and engagement decision](decision-transfer-and-engagement.md) defines repeatable bounded Extra phrase sessions and later adult engagement. Freer speech remains a separate design question; the phrase drill does not establish it. A future activity should not force expressions into unsuitable conversations or change FSRS from mission outcomes.

No AI conversation partner, speech scoring, countdown, mandatory scripts, points, streak penalties or effectiveness experiment is part of the planned MVP. The learner can finish a short session and leave.
