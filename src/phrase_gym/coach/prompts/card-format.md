## Languages of this card

- Learning language: {{learning_language}}, stated level {{level}}. The `target`, `text`, the
  wording around each cue gap and the wording fragments in `recall_hints` are {{learning_language}}.
- Meaning-layer language: {{meaning_language}}. Write every cue gloss in `(=...)`, `explanation`
  and `usage_contexts` in {{meaning_language}}.
- Native language: {{native_language}}. Write `detailed_explanation` and an insufficient
  input's `reason` in {{native_language}}.

## Decide what to prepare

The input normally holds one {{learning_language}} expression or one word, optionally followed by
the sentence it came from. A separator such as a dash, colon or line break may divide them; the
learner may also omit it.

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
`insufficient` only when no coherent {{learning_language}} card can be prepared, for example when
the input has no {{learning_language}} word or expression, mixes several unrelated targets, or is
unreadable. Then give a short concrete `reason` addressed to the learner that says what went
wrong and what to do, such as "No word or expression in {{learning_language}} was found. Type a
word or phrase in {{learning_language}}.", written in {{native_language}}. The learner sees the
reason next to their text, so do not quote the input in it. Set `card` to null. Otherwise set
`outcome` to `ready`, `reason` to an empty string and fill `card`.

## Card fields

- `text`, `explanation`, `cues`, `recall_hints`, `usage_contexts` and `detailed_explanation`
  follow the content rules below.
- Every cue `situation` contains exactly one `__________` gap (ten underscores) hiding the
  whole target. Use `partner_line` for an optional short line by the other speaker, or null.
- `detailed_explanation` is null unless longer help is useful, such as a contrast with a
  confusable word or expression.
