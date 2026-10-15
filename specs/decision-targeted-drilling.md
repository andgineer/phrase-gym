# Product scope, prompts, words and help

Phrase Gym trains selected words, fixed expressions and prepared dialogues toward ready spoken use. It is a personal trainer, with no open-ended conversation, AI role-play or learner effectiveness experiments.

Each section below ends with the evidence it rests on. Summaries, limits and how each source was checked are in [research](research/README.md). No study tests this complete workflow: these rules apply the cited findings and are design inferences.

## What is being retrieved

The response is the whole phrase. Fixed expressions have declared acceptable equivalents; word/frame targets identify the required sense or construction and the slots that may vary. Different meanings are different targets. Examples, prompt variants, lesson memberships and dialogue appearances share one target and schedule.

## Prompts for fixed phrases

A prompt gives what a real conversation gives: a short situation and, where one exists, the other person's preceding line, played and shown. No part of the answer appears before the learner responds. The prompt is in the language being learned, never a translation. The situation carries what separates the phrase from the learner's other phrases for the same moment (register, degree, whether a question is returned), so that only this phrase fits among the learner's phrases in that language. A situation that several learned phrases fit is not a prompt.

Each phrase has several prepared prompts, rotated across reviews. A model writes them, an independent model review checks each against the learner's whole phrase set, and the learner accepts them during external preparation. A phrase without an accepted prompt stays available for study and lookup only.

Evidence:
- Speed gains did not transfer between practice modes, so automatic use needs practice in the mode of use: [Li & Taguchi 2014](https://experts.nau.edu/en/publications/the-effects-of-practice-modality-on-pragmatic-development-in-l2-c/), [Morris, Bransford & Franks 1977](https://doi.org/10.1016/S0022-5371(77)80016-9). A native-language prompt exercises the translation route that proficient learners bypass ([Kroll & Stewart 1994](https://doi.org/10.1006/jmla.1994.1008)). See [match practice to use](research/phrase-prompts.md#match-practice-to-use).
- Research elicits conventional expressions with a situation plus the interlocutor's turn as audio, answered aloud, and instruction with oral practice increased their use: [Bardovi-Harlig, Mossman & Su 2017](https://scholarspace.manoa.hawaii.edu/server/api/core/bitstreams/12991145-c5d6-4eb5-b7c4-d2a228eb8d84/content), [Bardovi-Harlig, Mossman & Vellenga 2015](https://doi.org/10.1177/1362168814541739). Recognition alone does not produce use ([Bardovi-Harlig 2009](https://doi.org/10.1111/j.1467-9922.2009.00525.x)). See [situation plus partner turn](research/phrase-prompts.md#situation-plus-partner-turn-is-the-research-standard-prompt-for-routines).
- A cue works when it excludes competitors ([Poirier, Nairne et al. 2012](https://openaccess.city.ac.uk/id/eprint/2519/)). Collocation research constrains answers for exactly this reason ([Li & Boers 2025](https://doi.org/10.1111/modl.70007)). See [the prompt must identify one phrase](research/phrase-prompts.md#the-prompt-must-identify-one-phrase).
- Varied retrieval cues beat a repeated one, and learners misjudge this: [Butowska-Buczyńska et al. 2024](https://doi.org/10.1073/pnas.2413511121). See [vary the prompt](research/phrase-prompts.md#vary-the-prompt-a-fixed-fragment-becomes-a-crutch).
- Model-generated learning items are workable with a review step ([Attali et al. 2022](https://doi.org/10.3389/frai.2022.903077)); no study measures model-written phrase prompts. See [model-prepared prompts](research/phrase-prompts.md#model-prepared-prompts).

## Hints, first meeting and repetition

Hint is available only on request. It first reveals a distinguishing word, never the opening words, which are rarely distinctive and become a crutch; further requests reveal more. A response after a hint is supported practice.

Before a phrase's first recall, the learner hears it, reads it with its situation and says it along with the reference twice. This study step is not graded. The app never asks for a phrase the learner has not studied. After each attempt, the comparison ends with the reference and the learner says the phrase once more with it.

Evidence:
- Cues withdrawn over trials gave the best final uncued recall ([Finley et al. 2011](https://bjorklab.psych.ucla.edu/wp-content/uploads/sites/13/2016/07/Finley_Benjamin_Hays_RBjork_Kornell_2011.pdf)). Less cue support during practice improved later recall ([Carpenter & DeLosh 2006](https://pubmed.ncbi.nlm.nih.gov/16752591/)). Fading support helps weakly learned items ([Fiechter & Benjamin 2018](https://experts.illinois.edu/en/publications/diminishing-cues-retrieval-practice-a-memory-enhancing-technique-/)). In the vault's 63 English phrases, 40 share their first word with another phrase. See [least help](research/phrase-prompts.md#least-help-that-still-succeeds-withdrawn-over-time).
- Studying before retrieving beat guessing, and guessed wrong forms reappeared despite feedback: [Strong & Boers 2019](https://doi.org/10.1111/modl.12579), [Strong & Boers 2018](https://core.ac.uk/works/9193838), [Boers et al. 2014](https://doi.org/10.1177/1362168813505389). Working with intact phrases worked best ([Boers, Dang & Strong 2017](https://doi.org/10.1177/1362168816651464)). See [study first](research/phrase-prompts.md#study-first-guessed-errors-persist).
- Saying material aloud improves memory: [Hashizaki](https://doi.org/10.1177/13621688231224040), [Forrin & MacLeod 2018](https://uwaterloo.ca/news/news/study-finds-reading-information-aloud-yourself-improves). See [say it aloud](research/phrase-prompts.md#say-it-aloud).

## Words

Every word item keeps the core of a flashcard: spaced retrieval of the link between word and meaning, with one schedule per word. Understanding and saying are stages of the same item, not separate cards. Two or three short contexts rotate across reviews.

What the card shows follows the learner's level in that language, a per-language setting the learner changes when ready:

- **Beginner** (German now): the front is a whole short chunk with audio, never a gap. The learner first practises understanding: hear and read the chunk, recall its meaning, then see the translation. Saying comes later: the translation prompts the whole chunk. Fixed expressions are whole items, and a reply can be prompted by the other person's line. Tapping the word shows its gloss with article and plural.
- **Independent** (English now): the front is a short context with the word missing, and the learner says the whole sentence. The answer shows a short definition in the help language and, where a lookalike exists, a contrast note (*further* versus *farther*, *non-* versus *un-*).

Two new confusable words are never introduced in the same session. Once both are known, mixed drills use contexts where only one fits. Serbian's level is not yet set.

Evidence:
- Flashcards and word lists gave larger gains than fill-in-the-blank and writing ([Webb, Yanagisawa & Uchihara 2020](https://doi.org/10.1111/modl.12671), per-activity results via [Lei & Reynolds 2022](https://doi.org/10.3389/fpsyg.2022.984211)). Flashcard-learned words become automatically processed ([Elgort 2011](https://www.lextutor.ca/freq/lists_download/elgort_2011.pdf)). See [flashcards versus context](research/words-in-context.md#flashcards-versus-context).
- A gap works only when nearly all other words are known, about 95–98% coverage ([lexical thresholds](https://nflrc.hawaii.edu/rfl/item-detail/206)). Shared Slavic words make Serbian readable early for a Russian speaker ([de Groot & Keijzer 2000](https://doi.org/10.1111/0023-8333.00110)). See [when a gap can work](research/words-in-context.md#when-a-gap-can-work).
- Several contexts beat one ([Bolger et al. 2008](https://www.lrdc.pitt.edu/perfettilab/pubpdfs/Context%20variation%20Bolger%20et%20al.pdf); [Butowska-Buczyńska et al. 2024](https://doi.org/10.1073/pnas.2413511121)). A single example sentence adds little, and receptive and productive practice build different knowledge ([Webb thesis](https://openaccess.wgtn.ac.nz/articles/thesis/Investigating_the_Effects_of_Learning_Tasks_on_Vocabulary_Knowledge/16945963); [Griffin & Harley 1996](https://www.cambridge.org/core/journals/applied-psycholinguistics/article/list-learning-of-second-language-vocabulary/F00040624B9C1370140CF398F9A1E0B0)). See [several contexts](research/words-in-context.md#several-contexts-practised-in-the-needed-direction).
- Translations help beginners most ([Prince 1996](https://doi.org/10.1111/j.1540-4781.1996.tb05468.x)). Stronger learners did relatively better with L2 definitions ([Joyce 2016](https://www.nepjol.info/index.php/NELTA/article/view/20205)). See [translation or L2 explanation](research/words-in-context.md#translation-or-l2-explanation).
- Similar forms are a systematic error source ([Laufer 1988](https://cris.haifa.ac.il/en/publications/the-concept-of-synforms-similar-lexical-forms-in-vocabulary-acqui/)). Related words learned together produce interference errors ([Nakata & Suzuki 2019](https://doi.org/10.1017/S0272263118000219)). Mixed practice helps discriminate similar categories ([Brunmair & Richter 2019](https://doi.org/10.1037/bul0000209)). See [confusable words](research/words-in-context.md#confusable-words).

## Help per language

Each learning language has its own help language and its own choice of whether the translation appears with the answer. English uses English help, no translation and no Russian. Its notes exist only where they add something (register, when the phrase fits, unusual words) and open from Explain. German shows the Russian translation with the answer and again after the recording comparison. Serbian uses no Russian; its help language is not yet set. Show & explain appears only when help exists in that language's help language; missing help has no placeholder.

Evidence: whether translation helps depends on proficiency, not on the language. See [translation or L2 explanation](research/words-in-context.md#translation-or-l2-explanation) and [Kroll & Stewart 1994](https://doi.org/10.1006/jmla.1994.1008). Choosing no Russian for English and Serbian is also the learner's preference. No study shows that an occasional native-language gloss harms an advanced learner.

## Keeping practice going

Game elements must themselves be useful practice:

- **Dialogue runs** play the learner's side of a prepared dialogue while the other lines play automatically. The pause for the answer shortens level by level, and a level is cleared without hints.
- **Progress:** each lesson shows how many of its phrases were recalled without help at their last review.
- **Real calls:** before a call, the learner can choose two or three phrases to use, and afterwards mark the ones used.
- **Weekly target:** replaces a daily streak.

There are no points, badges or leaderboards.

Evidence:
- Game elements have small positive effects ([Sailer & Homner 2020](https://doi.org/10.1007/s10648-019-09498-w)). Progress displays support the feeling of competence ([Sailer et al. 2017](https://opus.bibliothek.uni-augsburg.de/opus4/files/109059/109059.pdf)).
- A leaderboard with required badges lowered motivation and exam scores ([Hanus & Fox 2015](https://doi.org/10.1016/j.compedu.2014.08.019)). Expected rewards for activity undermine intrinsic motivation ([Deci, Koestner & Ryan 1999](https://leeds-faculty.colorado.edu/dahe7472/deci%201999.pdf)).
- The effect of game elements fades after about four weeks ([Rodrigues et al. 2022](https://doi.org/10.1186/s41239-021-00314-6)). Broken streaks cause drop-off ([Silverman & Barasch 2023](https://doi.org/10.1093/jcr/ucac029)).
- If-then plans improve goal attainment ([Gollwitzer & Sheeran 2006](https://doi.org/10.1016/S0065-2601(06)38002-1)).
- Speed needs more practice than accuracy ([Li 2013](https://benjamins.com/catalog/lllt.36.04li)), and repeating the same talk preserved fluency gains ([de Jong & Perfetti 2011](https://sites.pitt.edu/~perfetti/PDF/de%20Jong%20Language%20Learning.pdf)).

See [motivation](research/motivation.md) and [automatic use](research/automatic-use.md).

## Interaction constraints

One Start session action uses all lessons or a selected lesson. Starting explicitly replaces the known open session while retaining its history; Resume preserves the existing position. Browsing or changing language never replaces a session. Phrase lookup searches text and prepared concepts across the language's lessons, opens reference material and links to complete lesson lists. Browsing alone never grades recall.

Keep one compact practice card with the prompt, optional recording, Hint, answer reveal, inline help, replay and three self-check choices. Ratings save and advance quietly. The microphone directly starts another take. No recording-mode switch, problem submenu, Practice again, supported-only Continue, saved-rating banner or Undo UI.

Learning language belongs to bookmarkable URLs and scopes content/session views. UI follows either native or current learning language, with no independent UI-language selector. Explicit URLs win over best-effort device restoration; navigation makes no Settings writes. Settings saves one complete preference set at a time per tab, retaining failed drafts. Live cross-tab Settings synchronization is outside scope.

Automatic stopping, speech grading, historical recording-comparison UI and locked-screen/eyes-free operation are deferred.
