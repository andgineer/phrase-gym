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
Today’s practice ✦

🔥 8-day practice streak          Personal best: 12 days
Mon ● Tue ● Wed ● Thu ● Fri ● Sat ● Sun ●

TODAY · all lessons
18 completions · 6 first meetings · 2 skipped
6 Got it · 3 Effort · 1 Not yet
5 recordings available
[See today’s phrases]
So far, so good. · first meeting · Small talk
Another phrase · Effort · Small talk
[More]

YOUR WORK · all lessons
126 first meetings · 17 days with practice
Next return: Tomorrow · 4 phrases

[Daily] [Extra]
```

Today and Your work come from completed `practice_log` rows; Extra extends allowances without clearing them. The list shows completed phrases, including a wording snapshot after deletion. Saved-recording counts reflect recordings still available, while pending uploads are shown separately. Omit next return when none is scheduled. No due-change timeline or longest-gap highlight is claimed.

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

**Delete** on Library, phrase details or a revealed card back opens a confirmation naming the phrase and stating that the phrase, reference and saved recordings will be permanently removed. Cancel leaves the current screen and phrase view state unchanged. Confirmed deletion on a revealed card back advances without a grade or practice-log row. Confirmed deletion removes their visible records together and cleans up owned files, invalidates any current phrase view and prevents a pending upload from recreating the phrase or leaving an orphan recording. The pending local take shows an actionable deletion error until the learner discards it. Completed practice-log facts remain labeled as past activity, including for a phrase later deleted. There is no suspended state or Restore action.

## State, selection and transactions

- **Phrase and deletion.** `Phrase` is the complete record in `phrases`; `phrase.text` is its wording. Card is the visual client component, not another data entity. A phrase belongs to one source-scoped lesson and has prepared cues, at least two distinct one- or two-word `recall_hints`, one fitting searchable `usage_contexts` entry, explanations and reference. Validate the whole submission and publish lessons, phrases and references atomically. **Delete** removes the phrase, reference and saved takes after confirmation, while completed practice facts remain in `practice_log` with a wording and lesson snapshot and a nullable `phrase_id`. Pending uploads for a deleted phrase fail clearly; deletion never creates a practice completion.
- **Phrase schedule.** Keep FSRS JSON, due instant, first-study instant and nullable `last_review` UTC instant on the phrase. First-study Next schedules tomorrow's automatic first recall without setting `last_review`. On every explicit grade in Daily, Extra or phrase-specific Practice, compare `last_review` in the currently configured learner timezone with today. If null or another local date, call FSRS and atomically update JSON, due and `last_review`; if today, record practice without another FSRS call. Due dates filter automatic selection only. An explicit Practice grade can therefore reschedule a not-due phrase or a phrase first studied today. Show, Hint, Details, lookup, audio, recording, Skip and Extra entry never invoke FSRS.
- **Completed practice.** Use one `practice_log` row per completed first study, rated recall or Skip. No row is created by opening a card, changing cue, requesting Hint, revealing, recording, navigating away or Stop. A row has a stable completion ID for idempotency, nullable `phrase_id`, phrase wording and lesson snapshots, completion UTC instant and learning date, kind (first study or recall), nullable rating, Skip indicator, `fsrs_updated` and mode (Daily, Extra or Practice). One row is the source for Today totals/list, the current/best streak and cumulative activity. Do not add separate historical attempt/review tables, duplicate daily aggregate counters or FSRS snapshots to the log. Saved recordings keep their own cue/help/answer snapshots and stable take IDs without an attempt foreign key.
- **Daily settings and allowance.** Store `daily_due_limit` (default 20) and `daily_new_limit` (default five) in settings. They are independent category limits, so defaults permit up to 25 completions. Validate nonnegative integers and reject both zero. Read current settings on each selection and Daily entry; do not snapshot them into the session. Count today's Daily and Extra completed recalls and first studies from indexed `practice_log` rows; a skipped recall consumes a due slot. Phrase-specific Practice rows do not consume either allowance. A phrase merely opened or abandoned consumes none. If a setting increases, Daily can use the extra capacity immediately with the same counted completions; there is no sticky complete flag. The editor is later convenience work, while Stage 2 stores the settings.
- **Extra extension.** Keep two date-keyed allowance targets for today, one due and one new. For each category with configured limit `L` and today's completed count `C`, pressing **Extra** proposes `L` when `C < L`, otherwise `C + L`; store the maximum of that proposal and the existing target. Repeated presses at the same `C` do not compound. Selection uses the larger of the live configured limit and stored Extra target. A zero configured category is disabled, including on Extra; it receives no extension. For default limits: counts 12 due/2 new retain 20/5; 20/5 extends to 40/10; 27/7 extends to 47/12. A lower positive setting does not shrink a larger stored target. New learning dates begin with the live settings and no Extra target. Extra entry changes allowance targets and starts a fresh traversal; it does not itself choose a phrase or create a completion. Today's log and `last_review` remain the source of completed work and the grade guard.
- **Current view and selection.** Keep one mutable current-session row with ID, learning date, entry origin (Daily or Extra), lesson filter, current phrase/view phase, shown cues/hints and a bounded set of skipped phrase IDs for this traversal. Pressing **Daily** resumes this view, including one opened by Extra; if none exists today it opens today's view. A new date closes an unfinished old view ungraded. Pressing **Extra** replaces the view with a new ID, clears the traversal's skipped IDs and extends the date allowance as above. Fetch one phrase at a time: oldest due first while due count is below effective allowance, then new while new count is below allowance. Skip leaves a due phrase due but excludes it from this traversal; a later Extra press may offer it again. Do not cycle through not-due phrases. Opening phrase-specific **Practice** atomically closes any unfinished regular view ungraded and invalidates its ID, even when both views concern the same phrase; it keeps the traversal's skipped IDs and date allowance. Practice is the sole valid view until it returns to details. The next Daily entry selects the next eligible phrase with a fresh view ID rather than restoring the old view. Practice completions consume no due/new allowance. Changing lesson affects the next new view, not the current phrase. Stale view IDs conflict, and an exact retried cue/hint operation returns its prior choice.
- **Completion transaction.** A rating, Skip or first-study Next finalizes once and advances. One SQLite transaction checks the current view ID, writes the `practice_log` row with its completion ID, updates phrase FSRS JSON/due/`last_review` if this is the first grade today, and saves the next view state. Exact completion retries return the same result; changed payload under one ID or a stale unfinished view conflicts. Failed saves leave the same card with an actionable retry using the same ID and no partial work. A stopped take is retained locally before navigation; its upload can arrive later without changing the completion. Deleting a phrase removes its visible rows in one database transaction and cleans up owned files, reporting cleanup failure rather than claiming filesystem and SQLite atomicity.
- **Random support.** Each appearance chooses a random prepared cue, avoiding the phrase's last shown cue when another exists; store that last cue on the phrase because there is no attempt log. **New cue** and requested **Hint** choose another prepared value when possible. Current view state holds shown cue/hint values for reload and exact retry; a saved take snapshots support seen at capture. No cue/hint event log is needed.

These rules replace `/study/current` opening a Card on page load, `exposed_day`, separate attempt/review tables, suspension, the ungraded **Practise freely** action and awaited upload flushes on navigation. Keep selection, phrase scheduling, current-view mutation, library reads and atomic publication focused. The completion workflow owns its transaction; HTTP adapters do not open hidden writes. Vue views render state; the practice reducer stays pure, with route, preference and audio effects in focused adapters. Preserve the original visual design.

## Checkpoint 2A — Content and discovery

**Learner benefit:** import organized phrases, search by meaning/use and inspect or delete a phrase while existing practice stays usable.

- Extend input validation, fresh schema and atomic publication for lesson identities, hints and contexts. Add paged lesson choices, global phrase search and details with source/lesson, reference and paged saved recordings. Add confirmed **Delete** in Library and details; remove Suspend/Restore. Adapt existing practice to the `Phrase` record until 2B replaces its state model.
- Verify import rollback, identity conflicts, a 1,000-phrase mixed-lesson search, lookup without scheduling writes, Delete cancel/confirm, reference/take cleanup and pending-upload failure. A real-server mobile browser test exercises import, search, detail and Delete. Refresh contracts/types and run the ordered implementation gate, independent review and exact-commit CI.

**Stop after 2A:** discovery and deletion work end to end. This pre-release checkpoint may retain old practice tables temporarily while the existing path still uses them; 2B removes them without a migration requirement.

## Checkpoint 2B — Shared card and Daily

**Learner benefit:** choose a lesson and complete or resume today's due/new practice with honest ratings and optional recording.

- Implement `practice_log`, the date-keyed allowance targets, live daily settings, one mutable current view, phrase `last_review` and shared first-study/recall Card. Daily opens or resumes the same view; completion is recomputed from current limits and today's log. Keep first-study Next, requested Hint, random cues, Details, auto-advancing rating/Skip, revealed-back Delete, front/back microphone and background take saving. Remove historical attempt/review tables; saved recordings own their support snapshots.
- Test first study, first versus repeated same-day grade, FSRS/`last_review` atomicity, 20/5 independent limits, live setting increase after apparent exhaustion, log idempotency and absence of rows for open/abandoned views, stale-tab conflict, timezone change and failed-save rollback. Real-server Chromium/WebKit tests cover silent/recorded Daily, resume, reveal/rehearsal before rating, auto-advance, capture stopped on rating, late upload and mobile controls. Refresh contracts/types and run the ordered implementation gate, review and exact-commit CI.

**Stop after 2B:** Daily and Library work; Extra is not offered until its extension path is complete. A plain completion message is enough until 2D.

## Checkpoint 2C — Extra extension and phrase-specific Practice

**Learner benefit:** press Extra to extend today's due/new allowance, or practise one chosen phrase from details.

- **Extra** updates both date-keyed targets by the formula above and opens a fresh traversal with a new view ID; fetch phrases one by one under the effective allowances. It never resets completed counts. Stop returns to entry. If no eligible phrase remains, explain that immediately. Add phrase-specific **Practice** through the shared Card and return to details after rating, Skip or first-study Next. Any explicit Practice grade calls FSRS when `last_review` is not today, even before due.
- Test default 12/2, 20/5 and 27/7 examples; repeated Extra at unchanged counts; lower settings preserving prior positive targets; zero category; skipped due reappearing in a new traversal; no not-due cycling; stale old-view retry; due/not-due Practice; opening Practice while an unfinished Daily first-study view exists, including for that same phrase, rejects the old view's completion without a log row or allowance change; subsequent Daily selects a fresh view. Cover Stop and late takes. Real-server Chromium/WebKit tests cover entry-to-Extra, repeat press, Stop, detail Practice and mobile capture/replay. Refresh contracts/types and run the ordered implementation gate, review and exact-commit CI.

**Stop after 2C:** Extra starts only on an explicit press and never erases today's completed work.

## Checkpoint 2D — Completion and practice facts

**Learner benefit:** see today's completed practice, a truthful streak and phrases studied today.

- On an exhausted traversal, return to entry with a completion message. Show **Today** counts and a paged studied list from `practice_log` across Daily, Extra and Practice, with kind, rating or Skip and wording snapshot. Show available saved recordings and pending uploads separately; the latter can be replayed after upload. Keep lesson picker and global search usable. If a live setting increase or new eligible phrase creates capacity, the same Daily entry can continue; no stored complete marker blocks it.
- Show current/best streak and seven recent learning dates from distinct log learning dates. A day counts after any first study, rating or Skip, regardless of recording; open/reveal/abandon/Stop do not count. Before today's first completion, show the streak ending yesterday; after a missed day ends, current is zero. **Your work** uses cumulative log counts and practice days, including completed activity for a phrase later deleted. Next return uses current phrases' due dates; omit when none. No mastery score, FSRS history timeline or longest-gap highlight.
- Test day counts, paged Today list, deletion preserving log facts, Extra extension without duplicate completions, live-limit change, multiple/pending takes, timezone change and missed days. Real-server browser tests cover completion/entry/next-day transitions with search and Extra in both engines. Refresh contracts/types and run the ordered implementation gate, review and exact-commit CI.

**Stop after 2D:** points, missions and weekly targets belong to Stage 3. Real listening, language-content quality, Safari/headset and iPhone behavior remain separate attended acceptance.
