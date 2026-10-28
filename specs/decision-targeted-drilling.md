# Useful language and retrieval cues

Phrase Gym helps an adult bring useful expressions and word senses into their own speech. It complements courses, listening, reading and real communication. Retrieval on a card is an intermediate outcome, not evidence that a phrase was used naturally.

The first MVP accepts externally prepared English phrase batches. Freer speaking and adult engagement follow, before words from echo-words and further languages. The [delivery ladder](plan/implementation-plan.md) defines scope; the content rules below define what a supported card means.

## Phrase cards

Each card trains one useful expression for a clear communicative purpose. Preserve the course wording and source attribution; generated situations and explanations are identified as preparation, not quotations from the course. Prefer language the learner actually wants to use, grouped by real situations rather than a list of near-synonyms.

The first MVP supports fixed complete phrases. A variable expression can enter as a concrete complete example, without claiming to teach its whole construction. A variable-slot editor, dialogue graph and grammar syllabus are not required.

A prepared card contains:

- A stable source/item identity, learning language and complete target answer.
- A short explanation of meaning and appropriate use; useful accepted alternatives may be listed explicitly.
- One or two distinct, natural retrieval cues: a situation stating the intended message and optionally a preceding partner line. Prefer two when both are defensible; do not invent awkward wording to meet a quota.
- Reference speech matching the complete answer, prepared before the card is offered for study.
- Source attribution and the chosen card/help languages. An optional detailed native-language explanation can be shown on request; it is not required for every MVP card.

For the next phrase-practice stage, each phrase also has several hand-prepared recall hints of one or two words. They are brief aids for retrieving that particular wording among other natural replies, not extra descriptions of the situation or automatically extracted first/last words. **Hint** shows one randomly selected hint only when requested, avoiding an immediate repeat when another is available; a hinted attempt is supported and cannot receive an independent recall grade. The same short situation cues serve Daily and Extra, with available cues rotated; do not store a second set of situations that duplicates them. Keep the short meaning/use explanation on the card back and a detailed native-language translation and explanation in the detailed view. Neither explanation appears before an independent attempt.

For **So far, so good.**, the content fields are:

```json
{
  "answer": "So far, so good.",
  "recall_hints": ["far", "so … good"],
  "cues": [
    { "situation": "Colleague asks: How's it going?" },
    { "situation": "Friend asks: How's the new job?" }
  ],
  "explanation": "Things have gone well up to now.",
  "detailed_explanation": "«Пока всё хорошо». Короткий ответ о том, как идут дела сейчас. Он говорит о результате до этого момента, а не обещает, что дальше всё будет так же.",
  "accepted_variants": []
}
```

The hints and cues are examples of prepared content, not a rule to generate them mechanically.

During recall, prompts communicate an intention rather than paraphrase the answer word by word. They contain no target phrase, initial-letter code, masked answer or answer-bearing title. Natural shared topic words are allowed: a universal ban on every content word would make prompts obscure without research support. A partner turn is optional; openers need no fictional preceding line. Situations must be understandable without studying incidental vocabulary.

In the **So far, so good.** example above, each brief question can elicit that phrase but also other natural answers. The learner knows the task is to retrieve a learned expression; a requested hint can identify it more precisely while marking the attempt as supported. An underdetermined or misleading card can be suspended without a forgetting grade.

Avoid generating many synonymous cards for the same cue. For the MVP, use one chosen expression and a short accepted-alternatives note where appropriate; no preferred-target management system is required. Similar text alone never proves that two senses are the same.

**Evidence:** [oral pragmatic-routine instruction](research/phrase-prompts.md#situation-plus-partner-turn) provides a precedent for situation/partner cues and production. [Variable retrieval](research/phrase-prompts.md#vary-the-prompt) supports variation. Neither proves this cue format is superior to translation or that the app transfers to spontaneous conversation. The number of cues and fixed-phrase first scope are design choices.

## Words after the phrase MVP

Each word card teaches one intended sense through comprehensible chunks, a short definition and two or three different contexts. A raw echo-words entry needs enough context to choose its sense; otherwise the app asks for a short clarification instead of inventing one. Keep source identity so retrying the same entry does not duplicate it.

For English at the learner's stated B2 level, use a short English context with the target word missing, plus a sense cue if needed; ask the learner to say the complete chunk. For beginner German, use the Russian meaning of a short chunk and ask for the German chunk. Explain article/plural or another form only when relevant. Grade retrieval of the target sense/form, not an unrelated slip elsewhere in the sentence. Accepted alternatives must fit the intended meaning.

Use one productive schedule per word sense. Understanding a definition is not successful retrieval of the spoken form. Contexts and explanations follow [personal preferences](decision-personal-preferences.md); English-only help is a learner choice, not a universal teaching rule.

For troublesome near-synonyms or lookalikes, give a short accurate contrast, including overlap. Avoid introducing a batch of confusables together. No research establishes an exact prerequisite graph or later-day unlocking algorithm, so none is required.

**Evidence:** [word learning](research/words-in-context.md) supports deliberate form/meaning practice, productive retrieval and context variation. A gap is one convenient task format, not a demonstrated bridge to natural conversation. Two or three contexts is a practical default, not an experimentally established optimum.

## Transfer to active use

Spaced spoken retrieval is the MVP's contribution: make useful wording available from meaning, with feedback and varied cues. [Conversational-uptake evidence](research/automatic-use.md) supports instruction combined with meaningful speaking, including increased use in unscripted classroom conversations. It does not guarantee lasting everyday use or automatic speech for this app.

The [practice and engagement decision](decision-transfer-and-engagement.md) defines optional unlimited Extra phrase recall and later adult engagement. Freer speech remains a separate design question; the card drill does not establish it. A future activity should not force expressions into unsuitable conversations or change FSRS from mission outcomes.

No AI conversation partner, speech scoring, countdown, mandatory scripts, points, streak penalties or effectiveness experiment is part of the planned MVP. The learner can finish a short session and leave.
