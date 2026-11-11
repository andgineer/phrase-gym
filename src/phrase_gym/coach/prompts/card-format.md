## Decide what to prepare

The input normally holds one English expression or one word, optionally followed by the sentence
it came from. A separator such as a dash, colon or line break may divide them; the learner may
also omit it.

- One word, with or without its sentence: prepare a word-sense card (`kind` = `word`).
- A multi-word expression, with or without its sentence: prepare a phrase chunk card
  (`kind` = `phrase`) around that expression.
- A sentence alone: select its most useful reusable chunk and prepare a phrase card.

Set `target` to the exact chunk or word the learner must retrieve, as it appears in `text`.
When a source sentence is supplied, copy it into `text` unchanged, except for trimming
surrounding quotes or spaces. Without a source sentence, a phrase card's `text` is the
expression itself, written as the learner would say it, and a word card's `text` is one short
natural sentence using the word in the chosen sense. Coach speaks `text` aloud as the reference
audio, so keep it a complete, natural spoken unit.

Use the supplied context and your judgment to choose the sense or meaning. Set `outcome` to
`insufficient` only when no coherent English card can be prepared, for example when the input
has no English word or expression, mixes several unrelated targets, or is unreadable. Then give
a short concrete `reason` addressed to the learner, such as "No English word or expression was
found in “фывап”.", and set `card` to null. Otherwise set `outcome` to `ready`, `reason` to an
empty string and fill `card`.

## Card fields

- `text`, `explanation`, `cues`, `recall_hints`, `usage_contexts`, `accepted_variants` and
  `detailed_explanation` follow the content rules below.
- Every cue `situation` contains exactly one `__________` gap (ten underscores) hiding the
  whole target. Use `partner_line` for an optional short line by the other speaker, or null.
- `accepted_variants` lists genuinely suitable alternatives for the same meaning, or is empty.
- `detailed_explanation` is null unless longer English help is useful, such as a contrast with
  a confusable word or expression.
- Write all help in English.
