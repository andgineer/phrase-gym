# Phrase practice and adult engagement

The product goal is to expand the language an adult actually chooses while speaking, including useful expressions encountered in courses and videos. Independent card recall is an intermediate outcome. The app complements listening, reading and real conversation; richer language means appropriate expressive choices, not a quota of impressive phrases.

[Research on conversational uptake](research/automatic-use.md) supports deliberate phrase instruction combined with speaking practice, including gains in unscripted classroom conversations. It does not establish durable everyday transfer for this app or isolate one necessary exercise. Daily and Extra are repeatable bounded phrase sessions, not an unscripted speaking activity; the [delivery ladder](plan/implementation-plan.md) orders later work.

## Daily and Extra phrase recall

Use personally useful expressions with their meaning, communicative purpose, register and source. The first MVP represents complete examples. Daily and Extra use the same phrase card, randomly selected concise contextual gap cues, optional requested recall hints, answer reveal and recording/replay on both sides of the card.

Daily and Extra take the longest-due phrases and randomly chosen new phrases within today's allowances and offer them one at a time in random order, as the [core trainer selection rule](decision-core-trainer.md#daily-allowance) describes. Settings hold live daily due/new limits, initially 20 and five; completed counts come from `practice_log`. **Daily** resumes the current view and can continue after a settings increase. Each explicit **Extra** press extends today’s allowance once completed count reaches that category's base limit and starts a fresh traversal; repeating it without further completions does not add more. Extra does not erase completed work. Automatic selection ends when allowances or eligible phrases run out; it does not cycle through not-due phrases. The same phrase's schedule can change at most once per configured learning day through an explicit rating, even in phrase-specific Practice before due. Hint use does not suppress rating; the learner rates recall before help. New cards rated in Extra initialize FSRS immediately and return when due, no earlier than the next learning day.

The [chunk content decision](decision-targeted-drilling.md#phrases) defines what is retrieved:
a useful expression inside visible context, not the incidental details of a memorized sentence.
The learner recalls the missing chunk, optionally says the completed cue sentence, and rates
the chunk before requested help. The card back identifies it within the original source example.
Varied requested words or word fragments can nudge recall of the exact expression without
supplying the whole chunk; the cue gloss already supplies its meaning.

This is phrase retrieval in varied contexts. Reusing the cue and requesting a hint can help practise recall, but neither demonstrates freer speaking or natural use outside the app. A distinct freer-speaking activity needs its own functional design before scheduling it. No AI conversation partner, countdown, memorized whole script or compulsory target insertion is required.

## Evidence and history

Keep one `practice_log` row for each rated new attempt, later recall or Skip, with a date, mode, wording snapshot and rating/Skip outcome. It supports Today’s paged studied list and factual streak without a separate attempt or review history. Saved recordings keep cue/help snapshots; no FSRS state timeline is claimed.

On every explicit rating in Daily, Extra or phrase-specific Practice, call FSRS if the phrase's `last_review` is not today in the configured timezone; due date limits automatic selection, not this grade. Rating reflects the original attempt before answer reveal or requested help; needing a hint does not remove rating controls and can be reported honestly as **Again**. Search, More explanation, Show, Hint, recording, Skip and repeated same-day ratings do not create another schedule update. Another natural response to a broad cue is not automatically a failure of the target phrase.

Do not label expressions automatic, fluent or mastered from card intervals, phrase counts, weekly targets or recordings. Software checks verify these distinctions and saved state; they do not establish learning effectiveness. No mandatory learner experiment is a delivery gate.

## Practice entry and today's facts

Entry offers **Daily**, **Extra**, one unlabeled lesson field with the selection's due/new counts and the header’s **Library** button beside **Practice** and **Settings**. The entry panel comes first on the page with **Daily** and **Extra** at its top, so on a phone they can be pressed as soon as the app opens, without scrolling; the lesson field follows them. Vertical space is scarce on a phone, so the field carries no separate title or search box: empty, it hints **All lessons — select one**; opening it lists **All lessons** and the lessons, and typing narrows that list by lesson or source title. Leaving the field without choosing keeps the current selection, and the field names the selected lesson even when it is not among the loaded choices. Library opens a separate page with the same lesson field and a phrase search that ignores the selected lesson; for the same space reason the page has no title and the search carries only its **Search phrases or concepts** hint. Each result shows the phrase, its **Used for** contexts and lesson, and tapping it opens the phrase details, which hold **Delete**; results carry no buttons of their own. An empty library points to the header's **Coach** field; an empty lesson says **No phrases in this lesson**; a selection with nothing due or new says **Nothing waiting today** with its next due date when known. Neither mode starts automatically. A card shows its single mode, Daily, Extra or Practice, with today's progress; Extra cards offer **Stop**, which returns to entry without a completion.

An exhausted Daily or Extra traversal returns to entry with a completion message and a practice-facts panel below the entry panel. All facts are learner-wide across lessons and read from completed practice and current phrases; nothing stores a daily total or a complete marker, so a live limit increase, Extra or a new eligible phrase lets the same Daily entry continue. An entry left open refreshes its counts when the page returns to view; only a new learning day restarts the studied list and refreshes the lesson choices, so loaded pages survive a same-day return and a new day is never shown as today. A completion message is shown only on the learning date it appeared.

- The panel is laid out for a phone screen. Each group's title carries its leading numbers: "—" separates the label from them and "/" separates the numbers, while "·" separates the parts listed below. A part that is zero is left out.
- **Today — 20 recalls / 5 new** counts today's rated later recalls and new attempts across Daily, Extra and Practice. Below it come the ratings with any Skips, and the recordings made today that are still saved. Takes still waiting to upload from this browser are counted apart from Today, whatever day they were captured; a refused take keeps its own discard error instead. **See today's phrases** pages through today's completions in order, each with its wording snapshot, kind, rating or Skip, mode and lesson; a phrase deleted later stays listed and is marked deleted.
- **Progress — 3 days** gives the days with practice, then everything done in them: recalls, rated new phrases, the ratings with any Skips, and recordings still saved. It counts every rating and Skip, as well as preserved historical ungraded first meetings labeled separately, including those for phrases later deleted. A raw completion total is not shown because it mixes attempts and Skips.
- **Total — 111 phrases** describes the current library: how many are due now, rated or historically studied and due later, and not rated or studied yet. These match the **All lessons** counts, and due phrases beyond today's Daily allowance still count as due now. No date is named, because after a completed Daily a date would read as more work owed today, while Daily offers further due phrases only on a later learning day.
- **In long-term memory** counts phrases whose FSRS stability, the days until predicted recall falls to 90%, has reached 21 days, the interval at which Anki calls a card mature. It answers how many phrases are durably learned without naming weeks, and it is hidden while zero. A phrase rated **Again** loses stability and can leave the count until it is rebuilt.
- The streak follows [adult engagement](#adult-engagement).

Apart from the long-term memory count, no mastery score, per-phrase mastery label, scheduling-history timeline or longest-gap highlight is shown.

Phrase details show the answer, **Used for** contexts, short meaning, lesson and source, reference playback, the current due date, [saved recordings](decision-recording-and-comparison.md#observable-behavior), **More explanation**, **Practice** and **Delete**. Looking a phrase up never changes its schedule; **Practice** opens it on the shared card and returns to its details. Phrase search, lesson choices and saved recordings are paged with **More** and say when nothing matches.

## Adult engagement

Adults are the intended audience. [Motivation research](research/motivation.md) supports designing for choice, a sense of progress and meaningful activity, while separating enjoyment from persistence and learning. Engagement is planned product work, not an assumption that adult discipline makes repetitive drills sustainable.

Entry shows a visible **current practice streak** and a personal best, derived from the distinct learning dates of completed practice. A learning day counts when the learner completes a rating or Skip (or has a preserved historical first meeting) in Daily, Extra or phrase-detail Practice, regardless of rating or recording; opening, revealing, abandoning or Stop alone does not count. Each completion keeps the learning date of the timezone in force when it was completed, so a later timezone change affects only which day is today. Before today has practice, the current streak may still show the run ending yesterday; it resets after today ends without practice. Missing a day resets the current consecutive-day count but never erases the best streak or accumulated work. The streak has no effect on FSRS and is never used to shame the learner.

Choice, a clear finish and a sense of progress come from what practice already offers: Daily or Extra on a chosen lesson is a short bounded session with varied cues and a completion message, and the practice facts are an informative record of accumulated work. Progress stays factual, each figure with its own label, with no single invented fluency score.

### Weekly target

A strict daily streak is the one engagement element the evidence flags as risky: a broken streak leads people to drop off, less so when the break is repairable, and a single missed day does not harm habit formation. The weekly target is the forgiving counterpart to the streak.

- The learner chooses a modest number of practice days per week. Entry shows this week's practice days against it beside the streak. A practice day is counted exactly as for the streak; a week runs Monday to Sunday in the configured timezone.
- The learner can change the target at any time and pause it. No target is set until the learner chooses one, and practice is fully usable without it.
- A missed day leaves the week reachable. A missed or paused week leaves no debt: no shortfall is carried forward, no missed-week count is shown, and accumulated progress and the best streak are untouched. Returning after a break offers the ordinary Daily allowance.
- The target has no effect on FSRS, allowances or card selection, and is never used to shame the learner.

The target is a default to refine from actual use, not a scientifically optimal prescription.

### Not planned

There are no missions, points, achievements or other rewards. Expected rewards for completing a task can reduce intrinsic motivation while informative progress feedback does not; points raised enjoyment without reducing dropout; and gamification effects faded within weeks. A mission would only rename the existing lesson choice, so it is worth adding only as the vehicle for a designed freer-speaking activity. There is likewise no public leaderboard, competition, child-oriented presentation, virtual economy or streak-based penalty.
