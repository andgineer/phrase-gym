# Phrase practice and adult engagement

The product goal is to expand the language an adult actually chooses while speaking, including useful expressions encountered in courses and videos. Independent card recall is an intermediate outcome. The app complements listening, reading and real conversation; richer language means appropriate expressive choices, not a quota of impressive phrases.

[Research on conversational uptake](research/automatic-use.md) supports deliberate phrase instruction combined with speaking practice, including gains in unscripted classroom conversations. It does not establish durable everyday transfer for this app or isolate one necessary exercise. Daily and Extra are repeatable bounded phrase sessions, not an unscripted speaking activity; the [delivery ladder](plan/implementation-plan.md) orders later work.

## Daily and Extra phrase recall

Use personally useful expressions with their meaning, communicative purpose, register and source. The first MVP represents complete examples. Daily and Extra use the same phrase card, randomly selected concise situation cues, optional requested recall hints, answer reveal and recording/replay on both sides of the card.

Daily and Extra fetch oldest due phrases, then new phrases, one at a time. Settings hold live daily due/new limits, initially 20 and five; completed counts come from `practice_log`. **Daily** resumes the current view and can continue after a settings increase. Each explicit **Extra** press extends today’s allowance once completed count reaches that category's base limit and starts a fresh traversal; repeating it without further completions does not add more. Extra does not erase completed work. Automatic selection ends when allowances or eligible phrases run out; it does not cycle through not-due phrases. The same phrase's schedule can change at most once per configured learning day through an explicit rating, even in phrase-specific Practice before due. Hint use does not suppress rating; the learner rates recall before help. New cards studied in Extra first return for independent recall on the next learning day.

For **So far, so good.**, one cue is **Colleague asks: How's it going?** and another is **Friend asks: How's the new job?** The learner says the short phrase, can request one of its brief hints, then sees the phrase and short explanation on the card back. The full phrase is enough; the activity does not demand extra speech.

This is phrase retrieval in varied contexts. Reusing the cue and requesting a hint can help practise recall, but neither demonstrates freer speaking or natural use outside the app. A distinct freer-speaking activity needs its own functional design before scheduling it. No AI conversation partner, countdown, memorized whole script or compulsory target insertion is required.

## Evidence and history

Keep one `practice_log` row for each completed first meeting, rated recall or Skip, with a date, mode, wording snapshot and rating/Skip outcome. It supports Today’s paged studied list and factual streak without a separate attempt or review history. Saved recordings keep cue/help snapshots; no FSRS state timeline is claimed.

On every explicit rating in Daily, Extra or phrase-specific Practice, call FSRS if the phrase's `last_review` is not today in the configured timezone; due date limits automatic selection, not this grade. Rating reflects the original attempt before answer reveal or requested help; needing a hint does not remove rating controls and can be reported honestly as **Not yet**. Search, Details, Show, Hint, recording, Skip, repeated same-day ratings and mission completion do not create another schedule update. Another natural response to a broad cue is not automatically a failure of the target phrase.

Do not label expressions automatic, fluent or mastered from card intervals, phrase counts, completed missions or recordings. Software checks verify these distinctions and saved state; they do not establish learning effectiveness. No mandatory learner experiment is a delivery gate.

## Practice entry and today's facts

Entry offers **Search lessons**, which only narrows the lesson picker, **Lesson to drill** (default **All lessons**) with the selection's due/new counts, **Daily**, **Extra** and a global **Find a phrase** that ignores the selected lesson. An empty library keeps **Add material**; an empty lesson says **No phrases in this lesson**; a selection with nothing due or new says **Nothing waiting today** with its next due date when known. Neither mode starts automatically. A card shows its single mode, Daily, Extra or Practice, with today's progress; Extra cards offer **Stop**, which returns to entry without a completion.

An exhausted Daily or Extra traversal returns to entry with a completion message and a practice-facts panel above the entry controls, which stay usable. All facts are learner-wide across lessons and read from completed practice and current phrases; nothing stores a daily total or a complete marker, so a live limit increase, Extra or a new eligible phrase lets the same Daily entry continue. An entry left open refreshes its facts and lesson counts when the page returns to view, so a new learning day is never shown as today.

- **Today** counts completions, first meetings, Skips and each rating across Daily, Extra and Practice. It shows saved recordings captured today that are still available. Takes still waiting to upload from this browser are counted apart from Today, whatever day they were captured; a refused take keeps its own discard error instead. **See today's phrases** pages through today's completions in order, each with its wording snapshot, kind, rating or Skip, mode and lesson; a phrase deleted later stays listed and is marked deleted.
- **Your work** gives cumulative completions, first meetings and days with practice, including activity for phrases later deleted.
- **Next return** names the earliest learning date, not before today, on which a current studied phrase is due, with the number due by then. It is omitted when no phrase is scheduled.
- The streak follows [adult engagement](#adult-engagement).

No mastery score, scheduling-history timeline or longest-gap highlight is shown.

Phrase details show the answer, **Used for** contexts, short meaning, lesson and source, reference playback, the current due date, [saved recordings](decision-recording-and-comparison.md#observable-behavior), **Details**, **Practice** and **Delete**. Looking a phrase up never changes its schedule; **Practice** opens it on the shared card and returns to its details. Phrase search, lesson choices and saved recordings are paged with **More** and say when nothing matches.

## Adult engagement

Adults are the intended audience. [Motivation research](research/motivation.md) supports designing for choice, a sense of progress and meaningful activity, while separating enjoyment from persistence and learning. Engagement is planned product work, not an assumption that adult discipline makes repetitive drills sustainable.

Entry shows a visible **current practice streak**, a personal best and the last seven learning days, derived from the distinct learning dates of completed practice. A learning day counts when the learner completes a first meeting, rating or Skip in Daily, Extra or phrase-detail Practice, regardless of rating or recording; opening, revealing, abandoning or Stop alone does not count. Each completion keeps the learning date of the timezone in force when it was completed, so a later timezone change affects only which day is today. Before today has practice, the current streak may still show the run ending yesterday; it resets after today ends without practice. Missing a day resets the current consecutive-day count but never erases the best streak or accumulated work. The streak has no effect on FSRS and is never used to shame the learner. Optional points, achievements and a chosen weekly target follow later in the delivery ladder.

- Offer chosen missions using useful phrase practice with a clear finish and varied cues. Do not label repeated card answers as freer speech.
- Show concrete progress from durable daily totals and comparable saved recordings, each with its own label. Avoid a single invented fluency score.
- Let the learner choose a modest weekly target, pause and return without debt, penalties or losing accumulated progress. Short sessions have a clear endpoint.
- Offer optional points or achievements for completing useful practice, including honest unsuccessful attempts. Rewards never depend on choosing a favorable self-rating, repeating easy cards indefinitely or forcing a phrase into unsuitable speech. Turning rewards off leaves learning available.

Weekly targets and rewards are defaults to refine from actual use, not scientifically optimal prescriptions. There is no required public leaderboard, competition, child-oriented presentation, elaborate virtual economy or streak-based penalty.
