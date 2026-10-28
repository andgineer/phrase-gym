# Stage 2 — Daily and Extra phrase practice

The [delivery ladder](implementation-plan.md) selects the first remaining checkpoint below. Each checkpoint delivers a complete learner workflow in the retained Vue shell and removes its own entry when accepted. The [content](../decision-targeted-drilling.md#phrase-cards), [learning loop](../decision-core-trainer.md), [practice](../decision-transfer-and-engagement.md#daily-and-extra-phrase-recall), [recording](../decision-recording-and-comparison.md) and [application boundary](../architecture.md#application-boundary) decisions own lasting rules. This stage extends phrase recall, not freer speech. Before the first release, use a fresh application database and current import format; no migration or old-format support is required.

## Shared behavior and screens

Keep the original Vue shell, centered card, round microphone and responsive layout. The entry has **Search lessons**, **Lesson to drill** (default **All lessons**), selected lesson due/new counts, **Daily**, **Extra**, and global **Find a phrase**. Lesson search narrows the picker; phrase search ignores the selected lesson. Empty library retains **Add material**. A lesson with no phrases says **No phrases in this lesson**; no Daily work says **Nothing waiting today**, with the next due date if known. Neither mode starts automatically. **Daily** returns to its unfinished card through the same button. **Extra** is explicitly entered and shows **Stop** on its card.

A new phrase opens on its answer: phrase, short meaning, reference playback, optional recording, **Details**, and **Next**; there is no rating. On later appearances the front shows one short situation cue, **New cue** when another cue exists, requested **Hint**, microphone, **Show**, and **Details**. The answer, explanation, reference URL and accepted alternatives are withheld from the recall-front response. **Show** turns to the back; front-side **Details** turns to the same back with the longer explanation open. The back keeps the prominent round microphone for optional rehearsal before and after rating until **Next**, and shows the phrase, short meaning, **Details** until opened, **Phrase**, saved **Mine**/**Compare** when available, **Not yet**, **Effort**, **Got it**, **Skip**, and **Next**. A rating describes recall before help or reveal; requesting help never removes the rating buttons. Only one rating or Skip is accepted per appearance. **Next** completes a first meeting without a grade, or advances after a rating or Skip; it never silently rates. After Not yet or Effort, invite supported repetition before Next without creating a second grade. **Stop** or navigation may close an unfinished Extra attempt without a rating. Daily retains its unfinished attempt when the learner leaves the card. Suspending a phrase removes it from selection without a forgetting grade.

A stopped recording is retained durably in the browser before navigation; upload continues independently of route and rating, with pending/error recovery visible. The front can auto-play the learner's stopped take only; comparison with the reference starts only after reveal or in first study/details. The back can auto-play take then reference. Next and Stop do not await upload. Changing routes while capturing first stops and retains the take. Each take has its own stable ID. Uploads may arrive after rating, Next or Stop and remain attached to the original attempt, including an ungraded-closed one; they never create a grade or change scheduling. History associates every saved take with its attempt, even if that attempt is ungraded.

Phrase lookup shows the answer immediately, **Used for** contexts, short meaning, lesson/source attribution, reference, saved attempt history, **Details**, **Practice**, and **Suspend**/**Restore**. **Practice** opens that phrase through the shared card and returns to details after Next. Lookup itself never grades or changes scheduling; an explicit rating in Practice can update a due schedule. History labels Daily, Extra or Practice, cue and every requested hint, rating or Skip, due-date change, and each saved take. An unfinished attempt is not shown as completed practice. Search and history are paged; empty search says **No matches**.

### Screen examples

The counts and copy are examples. Keep the entry controls below the completion panel when it appears.

```text
Phrase Gym                         🔥 8-day practice streak
Search lessons [____________]
Lesson to drill [All lessons ▾]    3 due · 2 new
[Daily] [Extra]
Find a phrase [Search phrases or concepts____________]
So far, so good. · Used for: small talk · progress update
[Open details]
```

### Recall card front

```text
Daily · 4 / 20                    (Extra: [Stop])

Colleague asks: How's it going?

[New cue]    [Hint]

             [🎙]

[Show]       [Details]
```

### Recall card back

```text
Daily · 4 / 20                    (Extra: [Stop])

So far, so good.
Things have gone well up to now.

             [🎙 Record / stop]

[Details]    [▶ Phrase]    [▶ Mine]    [Compare]

How was your answer before you saw the phrase?
[Not yet]    [Effort]    [Got it]    [Skip]

[Next]
```

The microphone remains on the back after a rating until **Next**, so the learner can rehearse and compare again without changing the rating. The mode/progress line shows either Daily progress or Extra/Stop.

### First meeting

```text
Daily · 4 / 20                    (Extra: [Stop])

So far, so good.
Things have gone well up to now.

             [🎙 Record / stop]

[Details]    [▶ Phrase]    [▶ Mine]    [Compare]

First meeting · no rating
[Next]
```

A first meeting opens on this answer side; **Mine** and **Compare** appear after a take is recorded.

```text
Daily complete ✦
You practised 14 phrases today.    🔥 8-day practice streak
Mon ● Tue ● Wed ● Thu ● Fri ● Sat ● Sun ●
Personal best: 12 days
TODAY · Small talk
10 returned phrases · 4 new phrases
6 Got it · 3 Effort · 1 Not yet · 5 recordings saved
A MOMENT FROM TODAY
“So far, so good.” · Got it after 12 days   [▶ My take]
YOUR WORK · all lessons
126 phrases studied · 81 revisited on a later day
17 days with practice
Next return: Tomorrow · 4 phrases
```

Omit a highlight without a prior-day rated return, and omit the next-return line when no return is scheduled. Show pending uploads separately from saved takes. The ordinary entry and its selected lesson remain available below the panel.

```text
[Back]
So far, so good.
Used for: small talk · progress update
Things have gone well up to now.
[Practice] [Details] [▶ Phrase]
Lesson: [lesson title] · Source: [source label]
Next due: [date]
History
[date] · Daily · Got it · next due [date]
Cue: Colleague asks: How's it going?   [▶ Mine] [Compare]
[date] · Extra · Effort · practice, due unchanged
Cue: Friend asks: How's the new job? · Hint: far
[Suspend]
```

## State and policy

- **Content identity.** Every phrase has a stable internal ID and belongs to one lesson identified within its source; same lesson labels from different sources do not merge. Repeated source/lesson IDs keep the same title; a changed title conflicts until submitted under a new lesson ID. A submission can contain multiple lessons. `PhraseMaterial` is the typed phrase record; normal learner copy says “phrase.” Each phrase has at least two distinct, hand-prepared one- or two-word recall hints and at least one fitting, short, searchable entry in `usage_contexts`; the [example](../decision-targeted-drilling.md#phrase-cards) is the content reference. Validate a whole submission before publication, then commit lessons, cards and references atomically. Never silently overwrite an existing source/item identity.
- **Learning day.** Resolve the learner's configured IANA timezone when an attempt completes, and store its completion date, zone and instant. An attempt left open across midnight keeps its displayed cue/support, but completion and any review belong to the actual completion day. First study then returns no earlier than the next learning date. Daily budgets and streaks use completion dates. A timezone change never rewrites historical rows. To avoid a duplicate scheduled review caused by a timezone change, a card is ineligible when its last scheduled review falls on the current learning date in either the zone used for that review or the currently configured zone. The server checks this at rating time. “Tomorrow” means the next local learning date, not 24 hours.
- **Daily budget.** The 20 completed-card and five completed-first-study limits are global to Daily across all lessons for a learning day. Extra and phrase-detail Practice never consume these budgets. A rated, Skipped or completed first-study Daily appearance consumes one card slot; a first study also consumes one new slot. An opened or abandoned appearance consumes neither; the same unfinished Daily attempt resumes through Daily even after selecting another lesson, and the entry shows which lesson it belongs to. Changing the lesson affects only later selection. The opening of a card does not reserve a budget slot permanently. When the limit is reached, Daily is unavailable for more cards until the next day, while Extra remains available. Counts on entry show eligible due/new phrases for the selection, separately from remaining Daily capacity; they need not fall to zero when a limit is reached. Keep one small Daily counter row per learning date for completed cards and completed first meetings. Increment it atomically and once with finalizing a Daily rating, Skip or first-study Next; retries return the original result without another increment. The completion panel later summarizes actual attempt history for the learning day and selected lesson, not the counter.
- **Attempt lifecycle.** Keep at most one foreground attempt and one paused Daily attempt for this local app. Returning to entry or opening Extra/Practice pauses unfinished Daily; pressing **Daily** resumes that exact card even after a lesson change, with its original lesson shown, unless another mode selected the same phrase. Extra prefers another eligible phrase when possible. If Extra must select the paused Daily phrase, or the learner opens Practice for that phrase, close the paused Daily attempt ungraded in the same transaction that creates the foreground attempt. Its Daily counters stay unchanged; a stale Daily tab cannot finish it. Daily later selects eligible work normally. Never leave two valid first-study attempts for one phrase. Opening another Extra/Practice closes its unfinished foreground attempt ungraded. A resumed attempt does not create a duplicate. A finalized attempt stores mode, selected lesson filter, shown cue and requested hints, snapshots of all answer/support text actually displayed, completion date/zone/instant, and outcome (first study, rating, Skip, or ungraded close), including whether due changed. A rating is saved when pressed, atomically with any schedule change; after a rating or Skip, Next only navigates, while first-study Next completes that meeting. A failed rating leaves the attempt open with its prior state. An ungraded close can preserve exposure/takes without counting as completed practice. Suspending a card invalidates its open attempt. After restart or in a second tab, the server is authoritative; stale mutations return conflict and reload the current attempt. An operation ID identifies each practice-state mutation and its payload: exact retries return the original result, while reuse with different payload or a different rating/Skip/cue/hint/reveal operation after finalization conflicts. Cue/hint retries return the originally chosen value. Take uploads use separate stable take IDs and may be newly accepted or retried against finalized or ungraded-closed attempts; they never reopen practice state or affect counters, ratings or schedules. Avoid parallel client requests for one attempt.
- **Selection and scheduling.** Daily selects oldest due then new within its budget; Extra uses the same priority without the Daily budget. A due phrase that is Skipped remains due, but Daily does not reoffer it on that learning day. Extra postpones it behind other due/new and then other studied phrases, so it cannot starve the loop yet can recur after a traversal. Extra then considers other studied phrases by oldest last completed appearance and eventually cycles; exclude the just-shown or paused Daily phrase whenever another eligible phrase is available. The foreground/paused attempts and attempt history are enough for this ordering; no queue/cursor tables. Same-day rated practice is recorded but never changes FSRS after the first eligible scheduled review. A not-due rating also leaves FSRS and due date unchanged. A first study sets its first return for the next learning day; repeating it in Extra on the same day is supported practice, with a rating but no schedule update. The server tests eligibility at rating time. Suspended phrases never enter Daily or Extra.
- **Random support.** Each appearance randomly chooses a prepared cue other than the immediately previous shown cue if possible. **New cue** randomly chooses a different cue; **Hint** shows one random prepared hint other than the immediately previous shown hint when possible. Preserve the complete ordered list of cues and hints shown in the attempt so history remains truthful; retries do not draw again. Answer reveal records which answer, short explanation, detailed explanation and reference were exposed. The record need not snapshot unseen fields. Random choice does not require a generic distribution or queue engine.

These rules replace the current `/study/current` request that opens an attempt during page load, the `exposed_day` prohibition on same-day ratings, automatic advance on **Got it**, and awaited take-transfer flushes on navigation. Keep one shared scheduling policy and one transaction owner for each attempt mutation; selection, library reads and publication remain separate. The application boundary constructs services; the attempt workflow owns its transaction, and HTTP adapters expose its contract without opening hidden writes. Vue views render state; a pure practice reducer and focused route, preference and audio adapters handle orchestration and effects. Do not grow a catch-all application component or a general session engine.

## Checkpoint 2A — Content and discovery

**Learner benefit:** import organized phrase material and find a phrase by meaning or use, then continue using existing practice.

- Extend the current input, validation, fresh schema and atomic publication for lesson identities, `recall_hints` and `usage_contexts`. A source may contribute multiple lessons; maintain stable source/item conflicts and idempotent import retries. Update the external preparation instructions/example for the fields. Publish only when all references are ready.
- Add paged lesson choices and global phrase search across wording, cues, short/detailed explanations and contexts. Search **greetings** finds a greeting phrase without its exact wording. Phrase details show **Used for**, source/lesson, short/detailed explanations and existing reference/history; keep existing practice usable. Pagination also applies to detail history, with stable ordering and a **More** action. Search and details do not mutate practice.
- Cover malformed imports and atomic rollback, duplicate lesson labels across sources, idempotent retries, 1,000 mixed-lesson phrases, all search fields, pagination and read-only lookup. A real-server browser test exercises import, global search and details at mobile width. Refresh generated contracts/types and run the ordered implementation gate for this checkpoint; independent review and exact-commit CI remain required by the handout.

**Stop after 2A:** no new Daily/Extra card or schedule behavior is claimed. Its complete import/search/details workflow is deliverable while the existing practice path still works.

## Checkpoint 2B — Shared card and bounded Daily

**Learner benefit:** choose a lesson and complete or resume a bounded Daily session with truthful first study, help, self-ratings and recording.

- Restore the entry controls and selected-lesson counts. Implement global Daily budgets, active-attempt resume, first-study, front/back reveal, random cues/hints, Details, three ratings, Skip, Next and the shared card. Use the lifecycle and scheduling policy above, including atomic Daily counter updates; phrase-detail Practice may remain on its existing path until 2C. Preserve Library/Add material. Daily completion may show a plain message until 2D.
- Complete the browser audio behavior: retain takes before navigation, upload in the background, replay/compare after reveal, and recover pending/error takes. Keep recordings usable on both card sides. Use generated client types.
- Test day rollover, timezone changes, lesson changes, exactly-once Daily counters, paused/retried attempts, multi-tab conflicts, mismatched operation payloads, first study, due vs same-day/not-due ratings, randomized support, no answer leak, and failed save without fabricated grade. Real-server Chromium and WebKit tests cover silent and recorded Daily, resume, Details from front, Skip, post-reveal and post-rating back-side rehearsal, late upload after Next, and mobile controls. Refresh contracts/types and run the ordered implementation gate, review and exact-commit CI.

**Stop after 2B:** Daily and existing library remain usable; Extra is not offered until its complete loop exists. No completion facts panel is claimed.

## Checkpoint 2C — Extra and phrase-specific Practice

**Learner benefit:** continue phrase practice for as long as desired, or rehearse one phrase from its details, with the same honest ratings and recording path.

- Add the explicit **Extra** entry and **Stop**, unlimited due/new/other selection and cycling without immediate repeats. Resolve skipped due starvation using the shared selection rule. Add **Practice** from phrase details, including the first-meeting path and return to updated history. Replace the old ungraded **Practise freely** action. Suspended phrases are omitted; restore returns them without replaying old ratings.
- Use the same card, attempt transaction, due eligibility and audio behavior as Daily. Extra/Practice do not consume Daily capacity. Stopping or leaving an unfinished attempt records no grade; earlier ratings and takes remain. Empty selection stays on entry with a clear message.
- Test finite-library cycling, single-card repetition, paused Daily collision in Extra/Practice, Skip and Stop, same-day repeat, due Practice after lookup, suspend/restore, stale tabs and recording upload after leaving. Real-server browser tests cover Daily-to-entry-to-Extra, Stop, detail Practice, post-rating back-side rehearsal, late upload after Stop and mobile layout in both engines. Refresh contracts/types and run the ordered implementation gate, review and exact-commit CI.

**Stop after 2C:** Extra never starts automatically; completion facts and streaks remain for 2D.

## Checkpoint 2D — Completion and practice facts

**Learner benefit:** see what this Daily accomplished, a factual practice streak and the next scheduled return.

- When Daily reaches its capacity or eligible work for its selected lesson is exhausted, **Next** returns to entry with a completion panel for that learning day and selected lesson. Keep lesson selection, phrase search and Extra available. The panel remains on entry for that learning day and clears on the next. An intentionally left unfinished Daily attempt does not produce it. Persist a small date-and-lesson completion marker when Daily has no more eligible work or reaches its global limit. Derive panel counts from finalized Daily attempts for that date and lesson. No separate progress score or queue is needed. If Daily was completed for one lesson and the learner switches lessons with remaining global capacity, the panel reflects the most recently completed lesson and Daily can start for the new selection; global capacity still applies.
- Show **Today** for the completed learning day and lesson: distinct completed appearances, rated due returns, first meetings, each explicit rating and Skip, saved takes and separately pending uploads. A completed appearance counts once regardless of take count; Skip counts as completed but is not a rating. The panel's title names its lesson. Show a highlight only for an actually rated return after a prior-day appearance, choosing the longest gap with deterministic tie-break and optional saved take playback. **Your work** spans all lessons: distinct studied phrases, distinct phrases practised again on a later learning day, and distinct learning days with completed practice. The next-return date/count uses actual scheduled due dates for the panel's lesson and is omitted if none exists.
- Show current and personal-best streak plus seven recent learning dates on entry and completion. A day counts once if at least one first-study, rated or Skipped attempt in Daily, Extra or Practice completed; ungraded closes, reveal, lookup and recording alone do not count. Through the end of a day with no completed attempt yet, show the streak ending yesterday; after that day passes without practice, show zero current. Historical best and accumulated work persist. Attribute midnight-crossing attempts to their stored completion day and never recompute old days after a timezone change. Streaks never alter FSRS or reward a rating choice.
- Cover per-day and per-lesson totals, lesson changes, multiple takes/pending upload, skipped/ungraded attempts, long-gap highlights, day rollover, timezone changes and missed days. Real-server browser tests cover completion, ordinary entry, return on the next day and the still-available search/Extra controls in both engines. Refresh contracts/types and run the ordered implementation gate, review and exact-commit CI.

**Stop after 2D:** points, missions and weekly targets belong to Stage 3. Real listening, language-content quality, Safari/headset and iPhone behavior remain separate attended acceptance, not inferred from software checks.
