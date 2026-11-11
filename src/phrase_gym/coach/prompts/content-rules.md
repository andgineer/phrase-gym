The learning goal is to retrieve reusable wording in speech: for example, **strongly believe**,
**totally convinced**, **Trust me**, or **Here's another great thing about that**. An entire
sentence about AI or onboarding is a study example, not automatically the learning target.
A single word is prepared as one word-sense card, described after the phrase rules.

### Select the chunk before writing the cue

In each source example, identify the useful conventional expression and its boundaries.
Choose a coherent chunk that serves a transferable purpose, such as expressing confidence,
adding a benefit or referring to evidence. Keep its essential collocation and grammar together.
Do not select an arbitrary substring or make the learner reconstruct incidental product details.
Prefer one target per card; other useful expressions in the sentence can remain visible context.
A self-contained routine such as **So far, so good.** can itself be the whole target.

Keep `text` as the complete source example, unchanged; the card's reference audio speaks it. In
`explanation`, name the target chunk explicitly and explain its meaning/use briefly. Keep this
learner-facing help short; distinguish example context without repeating preparation boilerplate.
Do not add a `target_chunk` field or other undocumented fields.
A changed context in a cue is an authored exercise, never a quotation attributed to the source.

### Build a natural gap cue

Keep the exercise fast: a short conversational line with one `__________` gap, followed by a
plain-English meaning gloss in `(=...)`. The gap hides the whole selected chunk. Visible words
provide incidental context; an optional `partner_line` supplies the other person's short turn.
Do not replace this format with a scene description, role instructions or a paragraph to analyze.
For a complete routine, the response may be just the gap and gloss. Use a constant gap length,
not one underscore per letter or word. Do not reveal target initials or assemble part of the
answer in the cue body; requested wording support belongs in hints. A subject or auxiliary can
remain visible only when it is outside the selected chunk.

Prefer two natural contexts with different propositions that accept the same unchanged chunk.
One good cue is better than a forced second. Changing only a greeting or person's name is not
a different context; add a genuinely different conversational proposition or keep one cue.
For example, a greeting and "You've had a busy morning" provide different lead-ins to a day
question. Insert the chunk into each cue and read the result, including any partner turn:
grammar, punctuation, meaning and referents must all work. If a response refers to "the repairs"
or "your ticket", establish those details in the exchange instead of expecting the learner
to infer them from an unrelated or vague partner turn. A plausible guess is not an established
referent: make it explicit in the partner turn or omit that optional turn. Check the exchange
with speaker labels: the learner's "you" addresses the partner, and the partner's "you"
addresses the learner. A request checking whether the partner can do work must not follow a
partner turn assigning that work to the learner. Keep any follow-up question short. Another natural expression may fit; do not distort the cue to force
one unique English answer or label a sensible alternative a language error.

### Make the gloss immediately understandable

The gloss paraphrases the missing expression's meaning in ordinary English. It is not an
instruction to the learner, a grammatical analysis or an abstract label. Prefer `=How is your
day so far?` to `=Ask about today up to now`, and `=That can be tough` to `=Recognize a real
difficulty`. Use enough words to make the meaning clear; shortest is not automatically best.

Preserve the expression's intention and sentence frame: who acts, whose preference or opinion
is requested, whether the response is a question or a statement, and its time, polarity,
certainty and strength. A neutral inquiry about someone's day must not become `=Having a good
day?`; it does not assume a positive day. For **Would you rather we**, use `=Would you prefer
us to...` or `=Would you like us to... instead?` when the context supports an alternative.
The visible continuation supplies the proposed action or the alternatives. When only one new
option is visible, retain `prefer` or an explicit `instead` in the gloss; `Would you like us to...`
alone loses the comparison with the current plan. When both options are visibly joined by `or`,
that continuation already supplies the comparison. Do not replace this
frame with `=Which task...` or `=Which should we do?`: the latter also changes preference into
advice. Do not invent an alternative, condition or confirmation request absent from the exchange.
Keep meaningful distinctions in duration, eventual outcome and degree: a broad time expression
must not erase a long-horizon meaning, and a vague addition must not replace an emphatic one.
Keep the distinguishing meaning explicit in both glosses, including the one varied for context.
For **In the long run**, `=Over the long term` and `=Over a longer period` preserve the horizon;
`=Given enough time` only says that sufficient time must pass. Daily repetition in the visible
context does not supply the missing long-term meaning. Vary wording around the essential
meaning rather than replacing it with a broader nearby idea.
Paraphrase the chunk, not a new claim about a subject or action already supplied outside the gap.
For a connector, give a connector-like gloss rather than another complete content statement.

Vary gloss wording between the two cues when natural, while preserving the same intention.
For **This would allow you to**, `=This would let you` and `=You would be able to` are clear
alternatives. Do not paste one generic gloss into every context by default. Repetition is
acceptable for a very short routine when a different gloss would become strained or change
meaning. Common words shared with the target are not automatically a leak, but copying the
complete target or giving its sentence opening as a wording scaffold is not a meaning gloss.

The learner retrieves the chunk and may say the completed cue sentence. Self-rating concerns
that chunk's form and appropriate use, not the original example's incidental content. Revealed
`text` and audio remain the original source example; they need not repeat a cue's new proposition.
The API has one example/reference per card, not generated answer/audio pairs for every cue.
Do not create duplicate cards per context or claim automatic evaluation.

### Write varied hints for recalling the exact wording

The gloss supplies meaning. Each requested `recall_hints` entry must independently help the
learner reconstruct the studied wording. Normally combine two diagnostic words or short
fragments in a single hint, separated by ` / `: for **How has your day been so far?**, use
`day / been`. These are memory anchors, not a sentence opening or an answer to read aloud.
The anchors need not be in sentence order. Choose words that distinguish this expression;
`so far` alone, generic function words alone, abstract labels such as `speaker credibility`,
and another meaning paraphrase do not provide adequate wording support. Combining several
function words does not fix this: every ordinary hint needs a diagnostic content word, a
recognizable part of one, or a distinctive fixed marker. A pair like `that / but it` is still
weak. Test the second hint just as strictly as the first.

Before writing hints, identify the chunk's diagnostic lexical core (its useful verb, adjective,
noun or fixed idiomatic marker). Keep at least one such anchor, or a recognizable fragment of
it, in every ordinary hint. Do not replace all core wording with auxiliary verbs, pronouns,
time markers or location tails just to make the second hint different. For a longer question
about what keeps someone busy, `been / over there` loses the useful wording; `keeping / busy`
and `busy / over there` both retain it. Reusing the strongest anchor is better than inventing
a weak alternative. Check the weakest hint alone, hiding all the others, before accepting the
card. In short tense-focused routines, an auxiliary can itself be diagnostic, as in the day
question below; judge its role in this particular expression rather than banning auxiliaries.

Provide two to six genuinely different complete hints, each at most 40 characters. **Two hints
means two alternative memory aids, not two halves of one good hint.** For the day question,
`day / been` and `been / so far` are two alternatives. `day` and `been` are merely the first
hint split into weak pieces. Important anchors may appear in more than one hint; vary the
other support rather than forcing disjoint words. Reversing slash order, changing punctuation
or clipping the same word in two arbitrary places does not create a useful second hint.

Each entry must work on its own with every cue for that card. The app shows one randomly
selected hint on request and avoids an immediate repeat when possible. Never write an ordered
ladder, assume a previous hint was seen, or put complementary fragments in separate entries
that only help when combined. Do not pad the list with weak hints to reach the minimum.

Keep meaningful retrieval work after each hint. Never provide the entire selected chunk,
including all its words separated by slashes, even if the full source sentence is longer.
Do not supply an almost-complete response with only a trivial article or pronoun missing.
Check this after ignoring slashes: leaving out only "that" is not meaningful retrieval. Replace
one revealing anchor with a useful fragment or choose another pair so the learner must recover
substantial wording. Do this check separately for every hint, not just the first.
For short chunks, meaningful word fragments can preserve useful work: `sounds / f...` and
`that / fun` help retrieve **That sounds fun!**. For a two-word echo such as **Is it?**, lexical
pairs would reveal the answer; a compact construction reminder such as `be -> question` or
`echo "it's"` can instead help reconstruct it from the partner's statement. Reserve construction
reminders for such auxiliary echo replies. Other short expressions need a recognizable wording
fragment; an ordinary associated word can accompany it when a second target word would reveal
the answer. Do not turn hints into grammar puzzles such as `-ing opener`, `ability modal`,
`ends in to` or `-ing after preposition`. These require analysis rather than reminding the learner
of words. Prefer lexical anchors even when the expression includes a modal or a preposition.
Avoid meaningless slivers such as `i...` and `...t`. If two useful independent hints cannot be
written for the chosen target, flag that card for review rather than claiming arbitrary hints
are effective. Requested help remains supported retrieval, not successful unaided recall.

These are authored examples, not source quotations. Each hint column is a separate alternative:

| Selected chunk | Cue 1 | Cue 2 | Hint 1 | Hint 2 |
|---|---|---|---|---|
| How has your day been so far? | Thanks for coming, Maya. __________ (=How is your day so far?) | You've had a busy morning. __________ (=How has today been for you?) | `day / been` | `been / so far` |
| This would allow you to | We could send the agenda today. __________ prepare your questions. (=This would let you) | We could offer remote access. __________ work from home. (=You would be able to) | `would / allow` | `allow / you` |
| Would you rather we | __________ moved the meeting to Friday? (=Would you prefer us to...) | __________ updated the slides or rehearsed first? (=Would you like us to...) | `rather / we` | `would / rather` |
| That's not quite what we had in mind | __________. (=Not exactly what we intended) | __________. (=Our idea was a little different) | `quite / mind` | `not quite / had` |
| Can you tell me more about | __________ the scheduling problems? (=Please explain in more detail) | __________ the support your team needs? (=Could you give me more details?) | `tell / about` | `can / more` |
| How does that sound to you? | We could split the cost equally. __________ (=What do you think of that idea?) | I propose moving the meeting to Friday. __________ (=What's your reaction to that suggestion?) | `does / sound` | `sound / you` |

For the **not quite** examples, add a short proposed action as `partner_line`, such as
"We could replace the training with an email" or "Would a two-day workshop suit your team?".
The response is completely blank because the entire routine is the target. In the other
examples, visible context does not need to be memorized as part of the expression.

### Prepare a word-sense card

A word-sense card teaches one sense of one word through short comprehensible chunks. It uses
the same fields as a phrase card; there is no separate word structure or extra field.

Choose the intended sense from the supplied context, such as the sentence the word came from.
Without context, choose the most common useful sense for a B2 learner. Teach only that sense on
the card; another sense needs its own card.

- `text`: the supplied source sentence unchanged when there is one; otherwise one short,
  natural sentence that uses the word in the chosen sense. It is the revealed example.
- `explanation`: name the word and its part of speech, then define the chosen sense in plain
  English that is easier than the word itself, for example **reluctant** (adjective):
  unwilling and slow to do something because you do not want to. Mention article, plural or
  another form only when it matters for using the word.
- `cues`: two or three short contexts with different propositions. Each replaces the target
  word, in the form that context needs, with one `__________` gap and adds a short sense gloss
  in `(=...)` when the context alone would allow other words. The learner says the completed
  chunk; rating concerns the target sense and form, not unrelated slips elsewhere.
- `recall_hints`: meaningful partial forms with an ordinary associated word, such as
  `relu... / to admit` or `reluc... / unwilling`; never the complete word.
- `accepted_variants`: only words that fit the intended sense in every cue.
- `detailed_explanation`: for a troublesome near-synonym or lookalike, a short accurate
  contrast that includes their overlap, such as **reluctant** versus **reticent**.
- `usage_contexts`: short searchable uses, as for phrases.

Do not introduce several confusable words together on one card. Two or three contexts is a
practical default; a definition the learner understands is not yet spoken retrieval.

### Inspect every card before submission

1. **Useful target:** check that the selected chunk is worth reusing beyond this example.
   Check the boundaries; hide its complete lexical core rather than only an easy filler word.
   For a word-sense card, check that every cue, the definition and the example use one sense.
2. **Completion:** insert the target into every cue and read the whole sentence. Check grammar,
   naturalness, intended meaning and a coherent referent for words such as “that” or “it”.
   Read the gloss as a quick meaning cue: reject instructions, cryptic labels, changed intentions
   or a different question/frame. Compare both glosses; vary naturally without semantic drift.
3. **Discrimination:** try likely competing expressions for every cue.
   Check whether syntax and meaning distinguish them; allow natural alternatives.
   Assess how each requested wording hint helps identify the intended chunk without giving it all.
4. **Hints:** check every hint separately against every cue. Consider which part of the exact wording
   it should activate and what the learner must still recall. Reject full-chunk answers, repeated
   meaning glosses, generic labels, peripheral words alone and one useful pair split into separate
   entries. Check that every hint is a complete alternative, not a permutation or a dependent step.
   Check short-target exceptions carefully and correct weak hints.
5. **Transferable context:** keep the target constant across cues while changing useful context;
   do not demand memorization of the original sentence's business claim or technical vocabulary.
6. **Honest support:** identify the chunk or word in the explanation and keep suitable
   alternatives only. Do not append repeated disclaimers to learner explanations. Study precedes
   recall; requested help is supported retrieval.
