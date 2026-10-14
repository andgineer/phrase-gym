# P5 independent semantic review

Date: 2026-10-14. Reviewer: fresh delegated agent `p5_semantic`. This is a
text-content review, not operator approval, listening acceptance, an engineering
review round, or evidence of learner effectiveness. All real results remain
withheld pending operator human content and reference-listening approval.

## Scope and snapshot

Read `AGENTS.md`, the current start/handoff/guidance, `phrase-preparation.md`, the
measurement plan, both fixed source-fixture sets, both availability records, and
every available raw generation and advisory-review response, including responses
rejected before draft assembly. Judgments below use the actual learner prompt;
claims in private rationales cannot supply missing identifying information. Sufficient
identifying support is the criterion, not mathematical uniqueness among every
possible grammatical paraphrase. User-approved illustrative greeting masks are
plausible stored-phrase cues; their initial human judgment remains separate from
the model's incomplete competing-completion analysis.
No provider call, source modification, publication, or audio listening occurred.

The implementation coordinator confirmed the final immutable text-result snapshot
before closeout: 12 original and 12 revision-2 raw generations, with no further
text-provider calls or tuning planned. All 24 generation answers and all 13 raw
advisory-review answers were inspected. No case is pending or excluded.

| Measure | First run | Revision 2 | Total |
| --- | ---: | ---: | ---: |
| Concrete generations | 12 | 12 | 24 |
| Raw advisory-review answers | 4 | 9 | 13 |
| Schema-valid assembled advisory reviews | 4 | 8 | 12 |
| Jobs with conformance failure | 8 | 4 | 12 |
| Jobs reaching withheld `awaiting_review` | 4 | 8 | 12 |
| Proposed candidates inspected | 12 | 17 | 29 |
| Candidates with definite defects as submitted | 8 | 6 | 14 |
| Candidate masks requiring initial human judgment | 4 | 11 | 15 |
| Operator-approved or published real results | 0 | 0 | 0 |

Candidate-defect counts include both members of the failed comparable Serbian
mask set; they do not call every possible paraphrase a defect. The German
revision-2 job fails at advisory JSON schema, while its candidate belongs in
human judgment. The malformed original `long` answer has one readable candidate
included in the 29. Required-chunk, help and equivalent defects may prevent
package acceptance even where a mask itself remains plausible. Repaired content
is not measured content, and none of these judgments approves publication.

Snapshot hash evidence: for each folder, sort all `*.json` files by filename,
including `availability.json`; form UTF-8 lines `filename SHA256(file bytes)\n`;
SHA256 that concatenated manifest. The reviewed manifest hashes are:

- `p5-results/`: `70c8cd6622f02213dbe0d7c8829babea3518cfe9f916f1db2756d8a616e6b303`
- `p5-results-v2/`: `490b0296361a01e9ccb6e1e634bca090bf86140ce3137f94a05a7d4e32a251c2`

## First-run candidate decisions

Every row refers to the corresponding file under `p5-results/`. Each fixture
has one proposed candidate.

| Fixture / candidate | Conformance outcome | Independent semantic decision and concrete reason |
| --- | --- | --- |
| `day` / `cand-day-1` | Failed | Reject. Empty visible chunks; the afternoon situation permits both supplied competitors. An ongoing day does not force present perfect or the words `so far`. The private rationale invents that distinction. |
| `greeting` / `cand-1` | Failed | Reject. Empty visible chunks and punctuation-only chunks. `Fine, thanks. How about you?` fits the actual prompt; no identifying phrase anchor is displayed. |
| `spill` / `cand-1` | Withheld draft | Human judgment required. It preserves meaningful source surrounding text and hides the selected idiom. `let the cat out of the bag` / `give anything away` are serious same-intent competitors absent from the measured challenge. Its claim of exclusion is unestablished, but a paraphrase alone does not prove this known-phrase cue unusable. Literal word breakdown is less useful than idiom-level explanation. |
| `bank` / `bank-cand-1` | Withheld draft | Human judgment required. The actual phrase anchors select the geographic sense and hide the target without leaking it. `edge`, `side`, `shore` are legitimate competing words; review whether this learned phrase identifies the selected lexical objective sufficiently. The private exact-literary-slot claim is unsupported, and synonyms must not become automatic equivalents. |
| `short` / `cand-1` | Failed | Reject. Empty visible chunks. Thanking someone also permits `Danke schön!`, `Herzlichen Dank!` and `Besten Dank!`. The rationale claims the situation requests `Vielen`, although it does not. |
| `serbian-latin` / `cand-1` | Failed | Reject. Empty visible chunks. `iduće nedelje` remains a natural next-week alternative; the prompt does not request `sledeće`. |
| `long` / `cand-1` | Failed; malformed JSON | Reject. Extra closing brace prevents parsing. Even reading the content manually, `Had I known ... I would have taken ...` fits the counterfactual situation and both visible anchors. No reliable identification of the selected `If I had known` frame is established. |
| `anchor` / `cand-1` | Withheld draft | Human judgment required; promising noninitial anchor. `hearing from you` usefully distinguishes hearing from seeing. `eagerly anticipating` is a serious alternative that the weak measured challenge misses; it does not itself invalidate recall of the learned phrase. Before package acceptance, correct the required-chunk definition: only the middle is required despite the fixed-phrase requirement to mark all meaningful chunks. |
| `contraction` / `cand-1` | Withheld draft | Human judgment required. The ending is genuine source support and the frame is hidden. `I prefer to` is a serious same-intent competitor; two blanks do not themselves distinguish it. Review learned-phrase identification and contraction expansion separately instead of asserting unique grammatical completion. |
| `german-idiom` / `cand-1` | Failed | Reject. Only whitespace is nominally visible. The prompt invents a story involving a foreign country, which is absent from the source and unnecessary for the idiomatic sense. `Das kommt mir komisch vor` remains plausible. |
| `serbian-cyrillic` / `cand-1` | Failed | Reject. Empty visible chunks. Hoping for punctuality admits the source's negative competing sentence and many affirmative alternatives. The claimed affirmative requirement is absent from the prompt. |
| `missing` / `cand-1` | Failed | Reject. Invents agreement/acknowledgment context for source `Right.` with unspecified sense and null context. Empty visible chunks; `Sure`, `Okay` and `Correct` fit. This should have been an explicit no-reliable-cue outcome with unresolved meaning. |

## Revision-2 candidate decisions

Rows refer to `p5-results-v2/`. Candidate distinctions are actual different masks,
not merely reworded situations; different masks alone do not establish quality.

| Fixture / candidate | Conformance outcome | Independent semantic decision and concrete reason |
| --- | --- | --- |
| `day` / `cand-1` | Failed | Reject. Claims `How has your day` and `far?` visible but displays `_____ been so _____?`; the purported recalled `been so` is actually supplied. Its two ungrammatical competitors do not challenge the cue. |
| `greeting` / `cand-1` | Withheld draft | Human judgment required; matches the illustrative accepted mask design. It has meaningful known-phrase support and hides the beginning. `Fine` / `Not so bad` are real alternatives, so the rationale's uniqueness claim is unsupported; these do not automatically invalidate a learned-phrase cue. |
| `greeting` / `cand-2` | Withheld draft | Human judgment required; promising gratitude mask hiding a different useful chunk. `thanks` / `thank you` can be a legitimate narrow equivalent boundary. The measured challenge uses `alright` / `well`, rather than realistic gratitude alternatives such as `cheers`; improve the serious analysis without treating paraphrase possibility as rejection. |
| `greeting` / `cand-3` | Withheld draft | Human judgment required; matches the illustrative accepted ending mask design. `And yourself?` / `How's yours?` deserve serious boundary analysis. The private possessive-pronoun requirement is unsupported. Normalize the raw mask's extra visible `?` against the hidden chunk containing punctuation, so reconstruction is clear. |
| `spill` / `cand-spill-1` | Withheld draft | Human judgment required. Actual surrounding source words are visible and the whole selected idiom is hidden. `say anything` / `tell anyone` are real alternatives missed by the weak redundant supplied competitors. Review whether these anchors sufficiently identify the learned expression; do not rely on the generator's claim of unique idiom selection. |
| `bank` / `bank-cand-1` | Withheld draft | Human judgment required. Geographic sense and actual source anchors are preserved, with the selected word hidden. `edge`, `side`, `shore` remain serious lexical competitors. Empty equivalents correctly avoid broadening the objective; initial review must judge identification of this learned source phrase. |
| `short` / `cand-1` | Withheld draft | Human judgment required. `Dank!` is meaningful support, while `Herzlichen` / `Besten` are common serious alternatives. The rationale's modifier-length/formality claim is unsupported. This mask needs judgment alongside the more constraining reverse mask, not automatic rejection for any possible paraphrase. |
| `short` / `cand-2` | Withheld draft | Reject as submitted because the literal learner prompt ends `Vielen _____@`. Removing `@` would leave a promising one-variant gratitude cue; `Vielen Müll` and `Vielen Gruß` are poor same-intent competitors. No repaired result has been measured or accepted. |
| `serbian-latin` / `candidate-1` | Failed | Reject the submitted pair because the two masks hide unequal required-chunk counts. Consider this mask for human judgment after comparable-set repair: next-week support and agreed-meeting context are useful; `Čujemo se` is a real alternative and the private exact-verb claim is unsupported. |
| `serbian-latin` / `candidate-2` | Failed | Reject the submitted pair: it hides two required chunks versus the other mask's one, so the declared comparable-support rotation set cannot conform. After repair this useful alternate ending mask needs human judgment. `iduće nedelje` is a legitimate competitor; the rationale invents an explicit `sledeće` requirement absent from the cue. |
| `new-idiom` / `cand-1` | Withheld draft | Human judgment required. Meaningful actual source beginning/ending support is supplied and the idiom hidden. `wrap up`, `finish for today`, `stop here` deserve serious comparison; the strictly-confined-slot rationale is false. Those alternatives alone do not prove this learned-phrase cue unacceptable. |
| `new-long` / `cand-1` | Withheld draft | Human judgment required. A specific source tail including meeting move and Friday gives useful identification. `Can you confirm whether` / `Do you know if` are serious wording alternatives; test learned-phrase identity and the fixed-expression boundary rather than demand unique syntax. |
| `new-long` / `cand-2` | Withheld draft | Reject. The actual prompt says only changed meeting time; it does not expose Friday, a meeting move, or a specific new date. `the meeting has been moved to Monday` and `the schedule has changed` fit. Advisory approval imports hidden source facts into the learner cue. |
| `new-contraction` / `cand-1` | Withheld draft | Human judgment required. The specific source tail and appreciative meal context meaningfully identify the construction. `needn’t have` is a serious same-intent alternative omitted by the weak challenge, which includes the answer itself. Review that selected-frame competitor and expansion `shouldn’t` / `should not` separately. |
| `new-german` / `cand-1` | Failed advisory-review schema | Human judgment required. The visible reunion tail is real identifying support and all selected frame chunks are hidden. `Ich kann es kaum erwarten` is a serious alternative. The rationale incorrectly claims reflexive/`auf` grammar is visible when those words are hidden. The raw review omits required `source_id`, so its rejection never becomes a conforming advisory record. |
| `new-serbian` / `cand-1` | Withheld draft | Human judgment required. Specific station-question support is visible and the requested frame hidden. Formal `Možete li da mi kažete` / `Recite mi` fit a passer-by too; informal register is supplied only by the rationale. Human review must judge adequate identification and preserve the source's valid informal objective. English native-help defects require correction independently. |
| `new-missing` / `cand-1` | Failed | Reject. Claims the sole chunk `Fine.` visible while showing a blank and marking no target leak. There is no useful partial mask. It invents a how-are-you sense/context despite the unspecified null-context source. Missing-context uncertainty should remain explicit. |

## Native help, meanings, search and equivalents

Most selected senses are preserved, except the unsupported narrowing of `Right.`
and `Fine.` and the invented foreign-country detail for the German idiom.
Straightforward Russian help for day, greeting, bank, thanks, Serbian next-week
and Cyrillic punctuality phrases is useful and appropriately concise. Search
terms generally represent real intent beyond literal words: gratitude, farewell,
secret, email reply, preference, counterfactual and directions are useful concepts.
This is a semantic inspection of metadata, not proof that real app queries work.
Missing-context terms for `Right.` and `Fine.` inherit the invented sense and
must remain unapproved. `serbian greeting` for the farewell is broad, not a
fatal mistranslation. Registers are not consistently represented explicitly.

Concrete help defects remain:

- Original `contraction` and `long`, original German idiom, and revision-2
  `new-idiom` put usage in English despite native language `ru`.
- Revision-2 `new-serbian` has English breakdown translations; `da` as simply
  `that / to` lacks the useful construction explanation for `da` + finite verb.
- Revision-2 `new-german`'s Russian meaning `радоваться чему-то (в будущем)`
  preserves anticipation broadly, but useful construction help should distinguish
  future anticipation with `sich freuen auf` / `darauf` + infinitive. A natural
  rendering of anticipation such as `с нетерпением жду ...` would better convey
  the source usage; this is a help-improvement judgment, not a fabricated sense.
- Revision-2 `new-idiom` has breakdown items `call`, `it`, `a`, `day` with no
  explanation; these add no native help. Its Russian meaning itself is accurate.
- Original `spill` gives accurate Russian idiomatic meaning and usage, followed
  by a literal word breakdown that could distract from that idiomatic meaning.
  Revision 2 improves this to an expression-level Russian equivalent.
- Original German idiom's literal `приходит мне по-испански` is awkward and the
  standalone translation of `vor` as `перед` does not explain separable
  `vorkommen`; the idiomatic Russian meaning is otherwise reasonable.
- Revision-2 `new-contraction` usefully explains appreciative unnecessary
  effort, but `shouldn’t = should not` alone is an English expansion, and
  `have gone to trouble` omits part of the actual expression. Source appreciation
  should be distinguished from literal criticism in the useful native help.

Equivalents require review independently of cue quality. Original `day`'s
`How is your day going so far?`, `short`'s `Herzlichen Dank!`, German idiom's
`komisch` / `verdächtig`, both spill runs' synonym idioms, and revision-2
`new-contraction`'s `You needn’t have done so much.` do not preserve the selected
exact expression/frame; they broaden acceptance to competitors. Original
`anchor`'s `awaiting your reply` / `waiting to hear back` are fragments replacing
the selected expression and can fail the displayed sentence's syntax.
Original `bank`'s `riverbank` is a real geographic synonym but not automatically
the selected lexical objective. Original Cyrillic `стићи` for `доћи` and Latin
`iduće` for `sledeće` likewise need explicit objective review.

`thanks` / `thank you` and `I’d` / `I would` are plausible narrowly reviewed
equivalents when full phrase acceptance is intended; they are not evidence that
ambiguous masks work. Revision-2 `new-long` changes `Could` to `Can` and
`whether` to `if` in its equivalent, changing the source's selected fixed
wording. All measured slot dictionaries are empty; no variable-slot semantics
have been established by these measurements.

## Advisory review is evidence, not acceptance

The first run's four reviews approve only `anchor`; revision 2's eight conforming
reviews approve only `new-long` / `cand-2`. The first approval inadequately
challenges serious alternatives; the second imports hidden source information.
Neither is operator acceptance. Some rejections also impose an overly strong
unique-completion criterion on useful learned-phrase masks. Geographic-word
and preference alternatives are legitimate review concerns, while the raw
rejections are not automatically operator judgments. Some explanations are
inaccurate:

- Original `spill` calls a definition with a blank a visible target leak, even
  though no target words are supplied. Its assertion that `spill the beans about
  the surprise` is inherently unnatural is unjustified. The actual defect is
  idiom identity ambiguity and poor competing-completion analysis.
- Original `bank` calls ordinary gap filling a target leak; hiding the word
  `bank` is not a leak. Multiple synonymous completions are a real boundary
  concern for human judgment, not automatic proof of an unusable learned cue.
- Revision-2 `spill` calls `ruin the surprise about the surprise` a natural
  competitor. Better competitors appear in the independent table.
- Revision-2 `short` wrongly uses `Vielen Gruß` as a plausible gratitude
  competitor; the literal `@` is a real independent rejection reason.
- Revision-2 `new-contraction` argues that poor competing completions make the
  actual prompt unnatural. Those are different issues; `needn’t have` supplies
  the concrete same-intent challenge missing from its analysis.
- Advisory help/meaning maps omit the English-for-Russian help defects in
  several cases. Empty error maps do not certify native help.

The illustrative greeting beginning/ending masks remain plausible under the
accepted product decision. Their useful support should not be rejected solely
because natural language permits paraphrases. Human judgment, accurate support
records, and narrowly reviewed equivalent boundaries are still needed.

There is no measured successful explicit no-reliable-cue result. In particular,
both missing-context cases manufacture cues/senses. Revision 2 produces useful
different hidden spans in greeting, short and long-meeting examples, but their
unrepaired complete sets are not semantically accepted. Original `anchor`
demonstrates a useful noninitial anchor without establishing reliable exact
recall. Software mask selection, synthetic approval, or schema acceptance cannot
change these content judgments.

## Measurement limits and remaining acceptance

All observed text answers use `llmbroker` / `google-gemini-3.5-flash-lite`; pool
availability lists four usable providers and five keyed, enabled models, not
comparative quality measurements. The first snapshot has 16 returned text
answers, 32,193 reported tokens and per-answer latency 0.899–2.244 seconds.
The final revision-2 snapshot has 21 returned text answers, 41,578 reported
tokens and per-answer latency 0.771–2.681 seconds. Combined reported usage is
73,771 tokens across 37 text answers. These are provider-reported
token usage, not an invoice. Jobs record one attempt; no retry benefit is
established. The free-pool plan reports zero metered text cost.

Four first-run and eight revision-2 drafts record generated
Edge full-phrase audio identities. No listening was performed, and text cannot
certify pronunciation, script handling, pacing, correspondence or speech quality.
Every inspected job has null human review and null published import ID.

Initial operator human semantic review and reference listening remain required;
no real generated content is approved for publication by this report. The
revised six-case held-out set must remain held out for this run. Any later prompt
tuning needs another newly fixed held-out set. Future evidence must separately
establish useful masks, honest missing-context/no-cue behavior, native help,
reviewed equivalent boundaries, real concept lookup and stored rotation. These
are content/integration acceptance gaps, not learner-effectiveness experiments.
