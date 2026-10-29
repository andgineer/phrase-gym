# Stage 2 — Daily and Extra phrase practice

The [delivery ladder](implementation-plan.md) selects the first remaining checkpoint below. Each checkpoint delivers a complete learner workflow in the retained Vue shell and removes its own entry when accepted. The [content](../decision-targeted-drilling.md#phrases), [learning loop](../decision-core-trainer.md), [practice](../decision-transfer-and-engagement.md#daily-and-extra-phrase-recall), [recording](../decision-recording-and-comparison.md) and [application boundary](../architecture.md#application-boundary) decisions own lasting rules. This stage extends phrase recall, not freer speech. Before the first release, use a fresh application database and current import format; no migration or old-format support is required.

## Shared screens and behavior

Keep the original Vue shell, centered card, prominent round microphone and responsive layout. Entry has **Search lessons**, **Lesson to drill** (default **All lessons**), selected lesson due/new counts, **Daily**, **Extra**, and global **Find a phrase**. Lesson search narrows the picker; phrase search ignores the selected lesson. Empty library retains **Add material**. A lesson with no phrases says **No phrases in this lesson**. A selection with no due or new phrases says **Nothing waiting today**, with the next due date if known. Neither mode starts automatically.

A new phrase opens on its answer: phrase, short meaning, reference playback, optional recording, **Details** and **Next**, without a rating. On later phrase appearances the front shows one short situation cue, **New cue** when another exists, requested **Hint**, microphone, **Show** and **Details**. The recall-front response withholds the answer, explanations, accepted alternatives and reference URL. **Show** turns to the back; front-side **Details** turns to the same back with the longer explanation open. The back keeps the round microphone for rehearsal after reveal and before rating, short meaning, **Details**, **Phrase**, available **Mine**/**Compare**, **Not yet**, **Effort**, **Got it**, **Skip** and **Delete**. A rating describes recall before help or reveal. Rating or Skip saves and advances immediately; there is no post-rating screen or second **Next**. First-study **Next** completes ungraded. Daily and Extra open the next eligible phrase or return to entry; phrase-specific Practice returns to details.

Before navigation, stop any active capture and durably retain its complete take in the browser. Upload runs independently and never blocks rating, Skip, first-study Next or Stop; pending/error recovery remains visible. The hidden-answer front may auto-play the learner take only. After reveal or in first study/details, comparison may play the take then reference. Cancel old-phrase playback on navigation so late events cannot play on a new phrase. Each stable take ID can upload to its original phrase after the view moves on, without affecting scheduling or progress.

The Library list also offers **Delete** for each phrase. Phrase lookup shows the answer, **Used for** contexts, short meaning, lesson/source, reference, current due date, saved recording history, **Details**, **Practice** and **Delete**. Practice uses the shared phrase view for this phrase. Lookup never changes scheduling; an explicit rating in Practice can reschedule whenever `last_review` is not today, even if the phrase is not due. Recording history shows date, mode, cue/help and answer snapshots stored with each take, plus **Mine**/**Compare**. It does not claim to be a review or rating history. Search and recordings are paged, with **More** and **No matches** states.

### Screen examples

The counts and copy are examples. Keep entry controls below the completion panel when it appears.

```text
Phrase Gym                         🔥 8-day practice streak
Search lessons [____________]
Lesson to drill [All lessons ▾]    3 due · 2 new
[Daily] [Extra]
Find a phrase [Search phrases or concepts____________]
So far, so good. · Used for: small talk · progress update
[Open details]
```

### Recall front

```text
Daily · 4 / 20 due                (Extra: [Stop])

Colleague asks: How's it going?

[New cue]    [Hint]

             [🎙]

[Show]       [Details]
```

### Recall back

```text
Daily · 4 / 20 due                (Extra: [Stop])

So far, so good.
Things have gone well up to now.

             [🎙 Record / stop]

[Details]    [▶ Phrase]    [▶ Mine]    [Compare]

How was your answer before you saw the phrase?
[Not yet]    [Effort]    [Got it]    [Skip]
[Delete]
```

The microphone stays available after reveal and before rating. Rating or Skip advances at once. **Mine** and **Compare** appear after a take exists. The mode line shows either Daily or Extra, not both.

### First meeting

```text
Daily · 1 / 5 new                 (Extra: [Stop])

So far, so good.
Things have gone well up to now.

             [🎙 Record / stop]

[Details]    [▶ Phrase]    [▶ Mine]    [Compare]
[Delete]

First meeting · no rating
[Next]
```

### Entry after a session

```text
Session complete ✦
This session: 10 returned phrases · 4 new phrases
6 Got it · 3 Effort · 1 Not yet

🔥 8-day practice streak          Personal best: 12 days
Mon ● Tue ● Wed ● Thu ● Fri ● Sat ● Sun ●

TODAY · all lessons
18 completed phrases · 6 first meetings · 2 skipped
5 recordings available

YOUR WORK · all lessons
126 first meetings · 17 days with practice
Next return: Tomorrow · 4 phrases

[Daily] [Extra]
```

Session counts reset when Extra starts a new session or a new learning day begins; Today and Your work do not. Saved-recording counts reflect recordings still available, while pending uploads are shown separately. Omit next return when none is scheduled. No longest-gap highlight or per-rating history is claimed without review records.

```text
[Back]
So far, so good.
Used for: small talk · progress update
Things have gone well up to now.
[Practice] [Details] [▶ Phrase]
Lesson: [lesson title] · Source: [source label]
Next due: [date]
Saved recordings
[date] · Extra · Cue: Friend asks: How's the new job?
Hint: far    [▶ Mine] [Compare]
[Delete]
```

**Delete** on Library, phrase details or a revealed card back opens a confirmation naming the phrase and stating that the phrase, reference and saved recordings will be permanently removed. Cancel leaves the current screen and phrase view state unchanged. Confirmed deletion on a revealed card back advances without a grade or session-counter increment. Confirmed deletion removes their visible records together and cleans up owned files, invalidates any current phrase view and prevents a pending upload from recreating the phrase or leaving an orphan recording. The pending local take shows an actionable deletion error until the learner discards it. Aggregate practice-day facts remain and are labeled as activity completed, including for a phrase later deleted. There is no suspended state or Restore action.

## State, selection and transactions

- **Content identity.** Every phrase belongs to one lesson identified within its source; same titles from different sources do not merge. Repeated source/lesson IDs keep the same title; a changed title conflicts until submitted under a new ID. A submission may contain several lessons. `Phrase` is the complete record in `phrases`; `phrase.text` is its wording string. Card is the visual client component with front and back views, not a second data entity. Require at least two distinct hand-prepared one- or two-word `recall_hints` and one fitting short searchable `usage_contexts` entry; the [example](../decision-targeted-drilling.md#phrases) is the content reference. Validate the whole submission and publish lessons, phrases and references atomically. Do not silently overwrite source/item identity.
- **Phrase schedule.** Store FSRS JSON, due instant, first-study instant and nullable `last_review` UTC instant on the phrase. First study sets tomorrow's first recall without setting `last_review`. On every explicit grade in Daily, Extra or phrase-specific Practice, compare `last_review` in the currently configured learner timezone with today. If it is null or on another local date, call FSRS and atomically update its JSON, due instant and `last_review` to now. If it is today, save only practice statistics. Due instant filters automatic queue selection; it is not a guard on an explicit grade. Thus a same-day phrase-specific Practice grade after first study can call FSRS once even though automatic Daily/Extra will not reoffer that phrase today. Show, Hint, Details, lookup, recording and Skip never invoke FSRS. A day and its aggregate facts use the configured timezone at completion; old facts are not redated after a preference change. “Tomorrow” means the next local date, not 24 hours.
- **Bounded sessions.** Daily and Extra each select oldest due first, then new, with independent limits `session_due_limit` and `session_new_limit` stored in settings, defaulting to **20 due and five new** phrases (up to 25 completed appearances). Snapshot both at session start so later settings changes cannot alter an open session. Validate nonnegative integer limits and reject both zero. A due phrase skipped in a session remains due; keep its ID in that bounded session so it is not selected again there. No not-due cycling follows exhaustion. One mutable current session holds its ID, origin (Daily or Extra), lesson filter, limit snapshots, due/new counters, skipped-due IDs and current phrase-view state (cue/hints shown, reveal, phase and pending operation result). Pressing **Daily** opens or resumes the one current session, including an Extra-started session; show its origin and progress plainly. A completed session stays complete on further Daily presses. If a new learning day begins, the next Daily press starts that day's first session and closes any old active view ungraded. Every explicit press of **Extra** starts a new Extra session with zero due/new counters and replaces any unfinished current session. Starting the session does not select phrases in bulk; request the next eligible phrase one at a time. **Stop** returns to entry. Changing the lesson affects the next new session, not an already open phrase view. Phrase-specific Practice temporarily replaces the current view, leaving session counters and ID intact, then returns to details; an interrupted regular view closes ungraded. A new session ID makes stale tabs and old completion requests conflict. Stage 2 stores these settings; the editor follows in a later convenience stage. Keep only the mutable current session and a bounded last-operation receipt for retry; do not create an attempt or review history table under another name.
- **Selection after a reset.** A fresh Extra session may offer a still-due phrase again if it was skipped in the previous session. A studied not-due phrase is not selected automatically, but phrase-specific Practice may grade it. A newly studied phrase first enters the automatic queue tomorrow. Counts on entry show eligible due/new phrases for the selected lesson; they may differ from the remaining slots in an open session. If none are eligible, the new session ends immediately with **Nothing waiting today**, without fabricating progress.
- **Completion and idempotency.** A rating, Skip or first-study Next finalizes the current phrase view and advances once. In one SQLite transaction, update the phrase schedule and `last_review` when this is the first explicit grade of the configured learning day, increment the session due/new counter only for its selected Daily/Extra phrase, update date-keyed daily/lesson aggregates for completed phrases, first meetings, ratings and skips, and retain the operation ID/payload/result receipt. Phrase-specific Practice updates aggregates but consumes no session slot. The session distinguishes due from new at selection; a new first meeting consumes a new slot, not a due slot. Session replacement leaves date-keyed daily aggregates intact. Failed writes change none of these; the same back remains with an actionable retry using the same ID. Exact retries return the prior result, mismatched payloads and stale session IDs conflict; duplicate clicks cannot double count. First-study Next follows the same rule. Stable take IDs upload independently after completion; recording rows carry `phrase_id`, date, mode, cue/help and displayed-answer snapshots directly, without an attempt foreign key. Delete phrase/reference/recording rows in one database transaction and remove their owned files as part of that operation; report cleanup failure instead of claiming filesystem and SQLite atomicity. Later uploads for the deleted phrase fail clearly.
- **Random support.** Each appearance randomly selects a prepared cue other than the immediately previous cue when another exists. **New cue** and requested **Hint** choose another prepared value when possible. Keep all shown cue/hint values in current phrase-view state while it is open; an exact retried operation returns the same choice. Saved takes snapshot the support actually shown at capture. Do not build a cue/hint event log or generic distribution engine.

These rules replace `/study/current` opening a Card on page load, the `exposed_day` block, attempt/review tables, suspension, the ungraded **Practise freely** action and awaited upload flushes on navigation. Keep selection, phrase scheduling, session mutation, daily aggregate updates, library reads and atomic publication focused. One session mutation workflow owns its transaction; HTTP adapters do not open hidden writes. Vue views render state; the practice reducer stays pure, with route, preference and audio effects in focused adapters. Preserve the original visual design rather than building a new shell.

## Checkpoint 2A — Content and discovery

**Learner benefit:** import organized phrase material and find a phrase by meaning or use, then continue using existing practice.

- Extend input validation, fresh schema and atomic publication for lesson identities, hints and contexts. Add paged lesson choices, global phrase search across wording/cues/explanations/contexts and phrase details with source/lesson, reference and paged saved recordings. Add confirmed **Delete** in Library and phrase details; remove suspended/restore controls and state. Adapt the existing practice path to the new Phrase record so it remains usable until 2B replaces its session model.
- Verify malformed-import rollback, identity conflicts, a 1,000-phrase mixed-lesson search, lookup without scheduling writes, and Delete cancel/confirm with reference/recording cleanup and pending-upload failure. A real-server browser test exercises import, search, detail and confirmed Delete at mobile width. Refresh contracts/types and run the ordered implementation gate, independent review and exact-commit CI.

**Stop after 2A:** discovery and deletion are usable; new Daily/Extra policy is not yet claimed. Since this is pre-release, a fresh database may remove obsolete attempt/review and suspension fields alongside the later practice replacement.

## Checkpoint 2B — Shared card and bounded Daily

**Learner benefit:** choose a lesson, complete or resume a short Daily session and rate recall honestly with optional recording.

- Implement the current-session state, phrase `last_review` marker, independent due/new counters, date-keyed aggregates and shared first-study/recall phrase view. Daily opens or resumes the one bounded session, including one started by Extra; it does not reset an exhausted same-day session. Keep first-study Next, requested Hint, random cues, Details, auto-advancing ratings/Skip, revealed-back Delete, front/back microphone and background take saving. Replace attempt/review tables; recording rows own their support snapshots.
- Cover first study, first grade today vs repeated same-day grade, FSRS JSON/`last_review` atomicity, timezone change, default and configurable independent due/new caps, session resume, stale-tab conflict, exact retry and failed-save rollback. Real-server Chromium/WebKit tests cover silent/recorded Daily, post-reveal rehearsal before rating, auto-advance, capture stopped on rating, late upload and mobile controls. Refresh contracts/types and run the ordered implementation gate, review and exact-commit CI.

**Stop after 2B:** Daily and Library remain usable. Extra is not offered until its complete bounded session exists. A plain completion message is enough until 2D.

## Checkpoint 2C — Resettable Extra and phrase-specific Practice

**Learner benefit:** press Extra whenever desired for another bounded due/new session, or practise one phrase from details.

- **Extra** starts a fresh session with zero due/new counters, replacing unfinished session state. Fetch its phrases one at a time with the same configured due-first/new-second caps and shared phrase view. No not-due cycle follows exhaustion. Stop returns to entry. If nothing eligible remains, explain that immediately. Add phrase-specific **Practice** through the shared phrase view and return to details after rating, Skip or first-study Next. Any explicit Practice grade calls FSRS when `last_review` is not today, even before the due date; otherwise it changes only daily statistics.
- Test repeated Extra presses, skipped due phrases reappearing in a new session, configured caps, no not-due cycling, stale old-session retry, unchanged daily aggregates on reset, due/not-due Practice grading with the same `last_review` guard, Stop and late takes. Real-server Chromium/WebKit tests cover entry-to-Extra, reset, Stop, detail Practice and mobile capture/replay. Refresh contracts/types and run the ordered implementation gate, review and exact-commit CI.

**Stop after 2C:** Extra starts only on an explicit press. Its repeated sessions retain daily totals.

## Checkpoint 2D — Completion and practice facts

**Learner benefit:** see the bounded session just finished, factual daily work and a current practice streak.

- Return to entry on final rating, Skip or first-study Next when the current session's configured due/new caps are reached or no eligible phrase remains. Show session counts from the current session and **Today** from durable date-keyed aggregates across Daily, Extra and Practice. Show saved recordings still available plus pending uploads separately. Today remains on entry for the current learning day; show the session summary when the current session is complete. Keep lesson picker and global phrase search below it.
- Show current and personal-best streak and seven recent learning dates. A day counts once with any completed first study, rating or Skip in Daily, Extra or Practice; ungraded navigation, reveal, lookup and recording alone do not count. Before today's first completion, show the streak ending yesterday; after a missed day ends, current becomes zero. Daily aggregates persist across session resets and phrase deletion. **Your work** shows cumulative first meetings and practice days from aggregates, labeled as completed activity even when a phrase was later deleted. Next return uses current phrases' actual due dates; omit it when none exists. Do not infer mastery or retain a longest-gap/per-review highlight without review history.
- Test session vs day totals, repeated Extra resets, deletion after practice, multiple/pending takes, timezone changes and missed days. Real-server browser tests cover completion/entry/next-day transitions with working search and Extra controls in both engines. Refresh contracts/types and run the ordered implementation gate, review and exact-commit CI.

**Stop after 2D:** points, missions and weekly targets belong to Stage 3. Real listening, language-content quality, Safari/headset and iPhone behavior remain separate attended acceptance, not inferred from software checks.
