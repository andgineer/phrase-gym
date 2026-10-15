# Phrase Gym review brief

Phrase Gym is a personal speaking trainer. We have just redesigned how it prompts phrases and words, which help it shows, and how it keeps practice going. Each decision cites studies. Please try to break the decisions, not confirm them.

**What we ask:** for each decision D1–D8, answer Keep, Change or Unsure, with your reason and, where you can, a source. If you know a better approach than ours, describe it: what the learner would see and do, why it should work better, and the evidence for it. A better alternative is more useful to us than a critique alone. The numbered questions are where we think we are weakest. Questions we didn't think of are the most valuable part of the review.

**Status:** proposed and written into the specs; nothing is implemented. The relevant repository files are:

- `specs/decision-targeted-drilling.md`: the decisions
- `specs/research/`: study notes
- `specs/decision-core-trainer.md`: scheduling
- `specs/plan/implementation-plan.md`: remaining work

**How each source was checked**, as tagged below:

- **[full text]**: we read the paper.
- **[abstract]**: we read only the publisher's abstract.
- **[secondary]**: another source described it.
- **[not read]**: we know it exists but did not weigh it.

## The learner and the app

- **Learner:** one adult whose native language is Russian.
- **English, B2.** The goal is to use standard conversational and business phrases automatically instead of improvising wording, and to learn rarer words (e.g. *further* vs *farther*, *nonconsensual*). The learner wants no Russian on English cards.
- **German, A1.** The learner needs translation even for basics. But they find one-to-one word cards ("Tür — ворота") misleading, and want words inside ready chunks such as "Sonst noch was?".
- **Serbian.** The learner wants no Russian; their level is not yet stated.
- **App:**
  - It runs in the browser. The learner speaks the answer, can record a take and compares it with reference audio (actor or synthetic).
  - The learner rates themselves Not yet / With effort / Recalled.
  - Scheduling uses FSRS at 0.90 desired retention, with one schedule per item.
  - There is no speech-recognition grading and no AI conversation partner.
- **First content:** 63 English phrases in 4 lessons from the learner's notes. They come from course phrase banks, with dialogues and audio.

## Decisions at a glance

| ID | Decision | Main support |
|---|---|---|
| D1 | Phrase prompts are in the language being learned: a situation plus the other person's preceding line. No translation, and no words of the answer shown. | Practice must match use; standard way of eliciting conventional expressions |
| D2 | A prompt must exclude the learner's other phrases. About three prompts per phrase rotate across reviews: a model writes them, a second model checks them against the whole phrase set, and the learner accepts them. | Cue discrimination; variable retrieval |
| D3 | Hint only on request, revealing a distinguishing word first and never the opening words. An answer given after a hint is "supported" and is not counted as recall. | Diminishing cues |
| D4 | Before a phrase's first recall, the learner hears it and says it along twice, ungraded. After each attempt the learner says it once more with the reference. | Study before retrieval; guessed errors persist; saying aloud helps memory |
| D5 | Words keep a flashcard core, with 2–3 rotating contexts and one schedule. The card format follows the learner's level in that language: beginners get whole chunks with a translation and no gaps; independent learners get a context with the word missing. | Flashcard effectiveness; coverage thresholds; varied contexts |
| D6 | Two new confusable words are never introduced in one session; once both are known, they are drilled mixed. | Interference; interleaving |
| D7 | Each language has its own help language: English help for English, the Russian translation with the answer for German, no Russian for Serbian. | Translation helps beginners most; learner preference |
| D8 | Game elements are practice themselves: dialogue runs with shrinking pauses, progress per lesson, choosing phrases before a real call, a weekly target. No points, badges, leaderboards or daily streak. | Gamification meta-analyses; studies of streaks and rewards |

---

## D1. Prompts in the language being learned

A phrase prompt gives what a real conversation gives: a short situation and, where one exists, the other person's preceding line, played and shown. The answer phrase is not visible before the learner speaks, and the prompt is never a translation.

An earlier proposal to prompt English phrases with their Russian meanings was withdrawn. It copied the translation format of memory experiments, and no evidence supported it for this goal.

Example card (English phrase):

```
You feel great today. Say so and ask back.
▶ "Hi Tom! How are you doing today?"
  [🎙]   [Hint]   [Show answer]

Expected: Really well, thanks. How about you?
```

Evidence:

- [abstract] [Li & Taguchi 2014](https://experts.nau.edu/en/publications/the-effects-of-practice-modality-on-pragmatic-development-in-l2-c/): 50 learners practised request forms with input-based or output-based training. Gains in accuracy transferred across task types; gains in speed did not.
- [secondary] [Morris, Bransford & Franks 1977](https://doi.org/10.1016/S0022-5371(77)80016-9): memory is best when the processing during learning matches what retrieval demands.
- [full text] [Bardovi-Harlig, Mossman & Su 2017](https://scholarspace.manoa.hawaii.edu/server/api/core/bitstreams/12991145-c5d6-4eb5-b7c4-d2a228eb8d84/content): routines were elicited with the topic and the intended move on screen, the classmate's turn as audio, and a 10-second spoken answer. Noticing plus oral practice raised oral use of the routines. The paper also summarizes [Bardovi-Harlig, Mossman & Vellenga 2015](https://doi.org/10.1177/1362168814541739), which found the same gains with oral practice.
- [secondary] [Bardovi-Harlig 2009](https://doi.org/10.1111/j.1467-9922.2009.00525.x): recognizing a conventional expression was necessary, but not sufficient, for producing it.
- [secondary] [Kroll & Stewart 1994](https://doi.org/10.1006/jmla.1994.1008), revised hierarchical model: beginners reach L2 words through L1; proficient speakers increasingly reach concepts directly.
- [not read] Counter-evidence we did not weigh: [Laufer & Girsai 2008](https://doi.org/10.1093/applin/amn018) report contrastive analysis and translation tasks as effective for learning vocabulary and collocations.

Questions:

- **Q1.1** Li & Taguchi studied request forms in Chinese over four days. Is "speed does not transfer across modes" enough to rule out translation prompts for a B2 learner whose goal is automatic use?
- **Q1.2** To single out one phrase, a situation must spell out register and degree ("You feel great today. Say so and ask back"). Is that just a paraphrase in the learned language that behaves like a translation prompt? Would it carry over to a real call?
- **Q1.3** Of the 56 phrases checked at the time, 32 had no preceding line in the dialogues; most are openers. Is a text situation a good enough trigger for openers?
- **Q1.4** Is it fair to lean on a model of word processing (Kroll & Stewart) for a decision about training prompts?
- **Q1.5** Does Laufer & Girsai 2008, or other work on translation, change the conclusion for spoken, automatic use?

## D2. Exclusive, rotating, model-prepared prompts

The situation must carry what separates the phrase from the learner's other phrases for the same moment, so that only this phrase fits among all the learner's phrases in that language.

Each phrase gets about three prompts, rotated across reviews: fewest presentations first, then the oldest. A model writes them, an independent model review checks each against the whole phrase set, and the learner accepts or rejects them before use. A phrase without an accepted prompt is available for study and lookup only, never drilled.

Evidence:

- [abstract] [Poirier, Nairne et al. 2012](https://openaccess.city.ac.uk/id/eprint/2519/): retrieval succeeds when a cue excludes competitors. A cue that fits many items fails even if it matches the target. This follows Nairne 2002 and Watkins & Watkins 1975 on cue overload.
- [full text] [Li & Boers 2025](https://doi.org/10.1111/modl.70007): study followed by cued gap-filling gave the best delayed collocation scores. The first letter was given "to prevent learners from supplying appropriate responses other than the target verbs."
- [abstract] [Butowska-Buczyńska et al. 2024](https://doi.org/10.1073/pnas.2413511121): varying the context sentence used as the retrieval cue beat a constant one, with or without feedback, after 24 hours. Learners believed constant cues worked better.
- [secondary] Goode, Geraci & Roediger 2008, and Butler et al. 2017: varied practice transferred better than repeating the same item.
- [abstract] [Attali et al. 2022](https://doi.org/10.3389/frai.2022.903077): reading items generated by a model, with human review, reached the Duolingo English Test practice test. No study measures model-written phrase prompts.

Questions:

- **Q2.1** Butowska-Buczyńska et al. used sentences as cues for recalling word translations, with retention over one day. Does that support rotating situation prompts for producing whole phrases?
- **Q2.2** Every new phrase can make an older prompt ambiguous. Can uniqueness against the whole set be maintained, or should another learned standard phrase simply count as correct?
- **Q2.3** Is three prompts per phrase the right number, given about 190 prompts for the learner to accept?
- **Q2.4** FSRS assumes an item's difficulty is stable. Do prompts of uneven difficulty make the self-ratings too noisy to schedule on?
- **Q2.5** Short of a learning study, what would convince you that model-written prompts are good enough?

## D3. Hints on request

A hint is never shown in advance. The first request reveals a distinguishing word ("well" in the D1 example), never the opening words; further requests reveal more.

An answer given after a hint counts as supported practice: the rating is kept, but it does not update the memory schedule. Among the 63 English phrases, 40 share their first word with another phrase and 16 share their first two words.

Evidence:

- [full text] [Finley et al. 2011](https://bjorklab.psych.ucla.edu/wp-content/uploads/sites/13/2016/07/Finley_Benjamin_Hays_RBjork_Kornell_2011.pdf): English–Iñupiaq word pairs. Without feedback, letter cues that shrank across trials gave the best final recall without cues: .40, against .25 for growing cues and .29 for study only. With feedback, both cue schedules beat study only. Retention was measured over minutes.
- [abstract] [Carpenter & DeLosh 2006](https://pubmed.ncbi.nlm.nih.gov/16752591/): fewer cue letters during practice gave better final recall.
- [abstract] [Fiechter & Benjamin 2018](https://experts.illinois.edu/en/publications/diminishing-cues-retrieval-practice-a-memory-enhancing-technique-/): for hard items without feedback, fading cues beat both plain testing and restudy. With feedback, fading cues and plain testing were equal.

Questions:

- **Q3.1** This learner always gets feedback, and with feedback Fiechter & Benjamin found fading cues no better than plain testing. Is a hint ladder worth the complexity?
- **Q3.2** "Distinguishing word first" is our own inference. Is there evidence on which part of a multiword phrase makes the best partial cue?
- **Q3.3** A phrase that always needs a hint never records a recall. Can it get stuck, and what should happen then?

## D4. Study step and saying it with the reference

Before a phrase's first recall, the learner hears it, reads it with its situation and says it along with the reference twice. This step is ungraded. The app never asks for a phrase the learner has not studied. After each attempt, the comparison ends with the reference, and the learner says the phrase once more with it.

Evidence:

- [abstract] [Strong & Boers 2019](https://doi.org/10.1111/modl.12579): studying phrasal verbs before retrieving them beat trial and error. 70% of the trial-and-error group's errors on the later test repeated their own wrong exercise answers.
- [abstract] [Strong & Boers 2018](https://core.ac.uk/works/9193838) (TESOL Quarterly, 140 learners): despite feedback, 25% of wrong exercise answers reappeared a week later.
- [abstract] [Boers et al. 2014](https://doi.org/10.1177/1362168813505389): matching exercises taught wrong verb–noun pairings despite feedback. [Boers, Dang & Strong 2017](https://doi.org/10.1177/1362168816651464): working with intact phrases gave the best results.
- [full text] [Metcalfe 2017](https://wesleyan.edu/ofcd/resources/OFCD%20Resource%20Docs/Learning%20from%20Errors.pdf): for typical learners, errors followed by correction can help, mostly with factual material. For phrase form, we sided with the phrase studies above.
- [abstract] [Hashizaki](https://doi.org/10.1177/13621688231224040): reading collocations aloud beat silent reading for recognizing form and meaning, including a week later.
- [secondary] Forrin & MacLeod 2018: reading aloud to oneself beat other study modes.

Questions:

- **Q4.1** The phrase studies used unfamiliar phrases that learners guessed before studying. This learner already met these course phrases. Is a study step for every new phrase justified, or wasted time?
- **Q4.2** Saying the phrase once more with the reference is echoing, not retrieval. Does it add anything, or does it dilute effort?
- **Q4.3** Is our reading of Metcalfe versus Strong & Boers right: errors can help with facts but persist in phrase form?

## D5. Words: flashcard core, card format by level

Every word item keeps the core of a flashcard: spaced retrieval of the link between word and meaning, with one schedule. Understanding and saying are stages of that one item, not sibling cards. Two or three short contexts rotate.

A per-language level setting chooses what the card shows:

- **Beginner** (German now): a whole short chunk with audio, never a gap. The learner first understands the chunk, with the translation shown alongside the answer. Later the learner says it, prompted by the translation.
- **Independent** (English now): a short context with the word missing. The learner says the whole sentence; the answer shows a short definition and, where needed, a contrast note.

Example cards:

```
German, beginner (understand)    ▶ Mach die Tür zu!
  answer:  Закрой дверь!
           die Tür, die Türen — дверь   (ворота — das Tor)
  later reviews rotate: Es klopft an der Tür. / Die Tür ist offen.

German, beginner (say)           Закрой дверь!   → Mach die Tür zu!

Serbian, independent             Čuješ kucanje.  ▶ „Ko je to?"
                                 Neko kuca na ____.
  answer:  Neko kuca na vrata.
           vrata — door (plural only: vrata su otvorena); gate — kapija

English, independent             Closed until ___ notice.   → further
  note:    farther = physical distance only
```

Evidence:

- [abstract] [Webb, Yanagisawa & Uchihara 2020](https://doi.org/10.1111/modl.12671), 22 studies: average gains were about 60% immediately, and 39% (meaning) or 25% (form) on delayed tests.
- [secondary] Per-activity results from the same meta-analysis, via [Lei & Reynolds 2022](https://doi.org/10.3389/fpsyg.2022.984211): flashcards and word lists gave relatively large gains; fill-in-the-blank and writing gave relatively small ones.
- [abstract] [Elgort 2011](https://www.lextutor.ca/freq/lists_download/elgort_2011.pdf): words learned from flashcards produced form, repetition and semantic priming, and were processed with a high degree of automaticity.
- [secondary] Hu & Nation 2000; Laufer & Ravenhorst-Kalovski 2010 ([lexical thresholds](https://nflrc.hawaii.edu/rfl/item-detail/206)): knowing about 95% of a text's words gives minimal comprehension, and 98% gives adequate comprehension. We apply this to single sentences.
- [abstract] [Bolger et al. 2008](https://www.lrdc.pitt.edu/perfettilab/pubpdfs/Context%20variation%20Bolger%20et%20al.pdf): varied contexts taught core meaning better than one repeated context.
- [abstract] [Webb, thesis](https://openaccess.wgtn.ac.nz/articles/thesis/Investigating_the_Effects_of_Learning_Tasks_on_Vocabulary_Knowledge/16945963): a single glossed sentence added little over a word pair. Receptive and productive practice built different knowledge.
- [abstract] [Griffin & Harley 1996](https://www.cambridge.org/core/journals/applied-psycholinguistics/article/list-learning-of-second-language-vocabulary/F00040624B9C1370140CF398F9A1E0B0): learning in the L1→L2 direction was more versatile when both comprehension and production are needed. Later work shows no consensus.
- [secondary] [de Groot & Keijzer 2000](https://doi.org/10.1111/0023-8333.00110): cognates and concrete words are learned more easily and forgotten less. We apply this to a Russian speaker reading Serbian.

Questions:

- **Q5.1** Please check the per-activity results in Webb et al. 2020 directly; we only have them second-hand.
- **Q5.2** The independent format still uses a missing-word prompt. We argue it is retrieval after study, unlike fill-in-the-blank used as a first-learning activity. Does that distinction hold?
- **Q5.3** The coverage thresholds come from reading whole texts. Is it fair to apply them to one prompt sentence?
- **Q5.4** Level is set per language, but difficulty varies by word. Should the format be chosen per item instead?
- **Q5.5** German saying is prompted by a Russian translation, while English phrase prompts must never be translations (D1). Is the difference by level principled, or inconsistent?
- **Q5.6** When should an item move from understanding to saying? This is not yet defined.

## D6. Confusable words

Two new confusable words (lookalikes or near-synonyms) are never introduced in the same session. Once both are known, mixed drills use contexts where only one fits, and the reason is shown after the answer.

Evidence:

- [abstract] [Laufer 1988](https://cris.haifa.ac.il/en/publications/the-concept-of-synforms-similar-lexical-forms-in-vocabulary-acqui/): words that differ by an affix or by one sound ("synforms") are a systematic source of learner errors.
- [abstract] [Nakata & Suzuki 2019](https://doi.org/10.1017/S0272263118000219), 133 learners: related words produced more interference errors, though overall scores did not differ significantly. The paper builds on Tinkham 1993, Waring 1997 and Nation 2000, which we know only second-hand.
- [abstract] [Brunmair & Richter 2019](https://doi.org/10.1037/bul0000209), 59 studies: interleaving beat blocking (g = .42), reportedly most when categories are similar. Most materials were visual categories.

Questions:

- **Q6.1** Overall scores did not differ in Nakata & Suzuki. Is the rule of introducing confusables separately worth enforcing?
- **Q6.2** Does interleaving evidence from visual categories carry over to word pairs such as *further*/*farther*?

## D7. Help language per language

Each learning language has its own help language, and its own choice of whether the translation appears with the answer:

- **English:** English help, no Russian. Notes appear only where they add something: register, when the phrase fits, unusual words.
- **German:** the Russian translation appears with the answer and again after the recording comparison.
- **Serbian:** no Russian; the help language is still open.

Evidence:

- [secondary] [Prince 1996](https://doi.org/10.1111/j.1540-4781.1996.tb05468.x): learning with translations gave better recall than learning from context, especially for weaker learners, while context helped transfer to new sentences. [Alroe & Reinders](https://api.equinoxpub.com/articles/34212) criticize its method.
- [abstract] [Joyce 2016](https://www.nepjol.info/index.php/NELTA/article/view/20205), 48 learners over 10 weeks: higher-ability learners did relatively better with L2 definitions, and lower-ability learners with L1 translations.
- No study we found shows that an occasional native-language gloss harms an advanced learner. The English-only choice is mainly the learner's preference.

Questions:

- **Q7.1** Given the evidence for translations, should Russian stay available on request for English words that an English definition fails to explain?
- **Q7.2** For Serbian without Russian, should help be in English, in simple Serbian, or both?

## D8. Keeping practice going

The chosen game elements are themselves practice:

- **Dialogue runs:** the learner speaks their side of a prepared dialogue while the other lines play automatically. The pause for the answer shortens level by level, and a level is cleared without hints.
- **Progress:** each lesson shows how many of its phrases were recalled without help at their last review.
- **Real calls:** before a call, the learner chooses 2–3 phrases to use, and afterwards marks the ones used.
- **Weekly target:** replaces a daily streak.

There are no points, badges or leaderboards.

Evidence:

- [abstract] [Sailer & Homner 2020](https://doi.org/10.1007/s10648-019-09498-w): gamification had small positive effects (cognitive g = .49, motivational .36, behavioral .25), and only the cognitive effect held in rigorous studies.
- [abstract] [Sailer et al. 2017](https://opus.bibliothek.uni-augsburg.de/opus4/files/109059/109059.pdf): badges, leaderboards and performance graphs raised the feeling of competence.
- [secondary] [Hanus & Fox 2015](https://doi.org/10.1016/j.compedu.2014.08.019): over 16 weeks, a leaderboard with required badges lowered intrinsic motivation, satisfaction and final exam scores.
- [abstract] [Deci, Koestner & Ryan 1999](https://leeds-faculty.colorado.edu/dahe7472/deci%201999.pdf): expected rewards for doing or completing a task reduced intrinsic motivation.
- [abstract] [Rodrigues et al. 2022](https://doi.org/10.1186/s41239-021-00314-6): the effect of gamification fell after about four weeks, then partly recovered.
- [abstract] [Silverman & Barasch 2023](https://doi.org/10.1093/jcr/ucac029): broken streaks led people to drop off, less so when the streak could be repaired.
- [secondary] [Gollwitzer & Sheeran 2006](https://doi.org/10.1016/S0065-2601(06)38002-1): plans of the form "if X, then I do Y" improved goal attainment (d = .65).
- [full text] [de Jong & Perfetti 2011](https://sites.pitt.edu/~perfetti/PDF/de%20Jong%20Language%20Learning.pdf): repeating the same talk under time pressure preserved fluency gains.
- [secondary] [Li 2013](https://benjamins.com/catalog/lllt.36.04li): about 4 practice rounds improved accuracy, but speed needed more than 8.
- [not read] [Thai & Boers 2016](https://doi.org/10.1002/tesq.232) reportedly found that repeating under rising time pressure improved fluency but not accuracy.

Questions:

- **Q8.1** The evidence comes from classrooms and consumer apps. Does any of it apply to one adult who chose to learn?
- **Q8.2** Shrinking pauses add time pressure. Could rushed wrong forms become fixed, like the guessed errors in D4?
- **Q8.3** Choosing phrases for real calls relies on self-report, and on research about planning actions rather than learning. Is it worth building?
- **Q8.4** Is there better evidence on what keeps a self-directed adult practising for months?

---

## Unchanged background you may also challenge

- **Scheduling:** FSRS 6.3.2 at 0.90 desired retention. The three ratings map Not yet to Again, With effort to Hard and Recalled to Good.
  - [abstract] [Kim & Webb 2022](https://doi.org/10.1111/lang.12479): spacing helps L2 learning, without one best interval ladder.
  - [abstract] [Sonbul et al. 2025](https://doi.org/10.1002/tesq.3364): equal and expanding spacing did not differ for collocation recall.
- **Grading:** the learner rates themselves after hearing their own take and the reference. There is no speech grading.
- **Out of scope:** AI conversation partners, shadowing as a memory method, keyword mnemonics, and large-volume reading or listening (left to other tools). Flag anything you think belongs in scope.
- **Excluded source:** Fang, Elgort & Chen 2024, which is retracted.

## Where our evidence is thinnest

- No study tests this complete spoken workflow; every rule is an inference from related findings.
- Many studies use word pairs or written collocations and small samples, with retention measured over minutes to weeks.
- Several key claims rest on abstracts or secondary descriptions: the per-activity results of Webb et al., Hanus & Fox, the lexical thresholds and Bardovi-Harlig 2009.
- The counts of phrases sharing first words, and of phrases with a preceding line, come from our own scan of the learner's notes.

## Response template

```
Reviewer:
Date:

D1 Prompts in the language being learned — Keep / Change / Unsure
   Reason:
   Source:
   Better alternative (what the learner sees and does, why, evidence):
D2 Exclusive, rotating, model-prepared prompts — Keep / Change / Unsure
   Reason:
   Source:
   Better alternative (what the learner sees and does, why, evidence):
D3 Hints on request — Keep / Change / Unsure
   Reason:
   Source:
   Better alternative (what the learner sees and does, why, evidence):
D4 Study step and saying it with the reference — Keep / Change / Unsure
   Reason:
   Source:
   Better alternative (what the learner sees and does, why, evidence):
D5 Words: flashcard core, card format by level — Keep / Change / Unsure
   Reason:
   Source:
   Better alternative (what the learner sees and does, why, evidence):
D6 Confusable words — Keep / Change / Unsure
   Reason:
   Source:
   Better alternative (what the learner sees and does, why, evidence):
D7 Help language per language — Keep / Change / Unsure
   Reason:
   Source:
   Better alternative (what the learner sees and does, why, evidence):
D8 Keeping practice going — Keep / Change / Unsure
   Reason:
   Source:
   Better alternative (what the learner sees and does, why, evidence):

Answers to numbered questions (e.g. Q2.2):

Evidence or risks we missed:

Alternatives to the overall approach (if you would design it differently):
```
