# Learning loop and observable evidence

The objective is easier access to useful language when speaking. This MVP loop measures practice and self-reported recall, not spontaneous conversation, pronunciation accuracy or mastery. Later [Extra practice and adult engagement](decision-transfer-and-engagement.md) build on it while keeping their outcomes separate from memory grades. [Research and its limits](research/README.md) distinguish empirical findings from the defaults below.

## First meeting

New phrases start with the complete phrase, a comprehensible meaning/use note and playable reference. Invite the learner to say it once; recording is optional. **Next** completes study without a grade or an invented successful recall. Importing a course phrase does not establish that the learner already knows it. The MVP uses this same short first meeting for every new phrase rather than inferring familiarity.

First automatic independent recall is offered on the next learning day. This is a simple spacing choice, not a research-derived optimum. Completing study today must not immediately reopen the same phrase through Daily or Extra as an independent test. Short sessions finish without a wait screen.

## Independent recall

1. Show a short situation and, when useful, the other person's preceding line. Hide the answer, reference audio, answer-bearing help and lesson/item titles that give it away.
2. Invite a spoken response, with an optional recording. Silent self-check remains available without microphone access. There is no countdown, speed score or required mock dialogue.
3. **Show** reveals the reference wording and short explanation; reference playback and comparison with the learner's take become available. **Details** also reveals the answer and opens the longer explanation. The learner rates the response made before help or reveal.
4. **Not yet** means missing/wrong wording or inability to retrieve without help; **Effort** means correct recall with difficulty; **Got it** means comfortable correct recall. A sensible alternative is not a language error: use the phrase's accepted alternatives, or delete a defective phrase from its detail view after confirmation.
5. The learner may rehearse with the back-side microphone after reveal and before rating. Pressing **Not yet**, **Effort** or **Got it** saves the rating and advances automatically; no second response or **Next** is required. Repetition, replay and a new take cannot replace the original response or produce another schedule update.

**Skip** closes the current phrase view without a rating and advances automatically. Revealing without an attempted response, inspecting an explanation, or a microphone/technical failure does not automatically mean forgetting. Requested Hint or Details does not hide rating controls; the learner can report what happened before using help. No AI judges speech or assigns a grade.

## Scheduling choices

Use the Python FSRS library with population parameters, desired retention 0.90, no short learning/relearning steps and no fuzzing. Map Not yet / Effort / Got it to Again / Hard / Good. Pin the selected library version and test integration; do not maintain a handwritten numerical port or require compatibility with prototype arithmetic fixtures.

At most one explicit grade per phrase per configured learning day calls FSRS and changes its schedule. Automatic Daily/Extra selection waits for the due date, but a learner-initiated phrase-specific Practice grade can reschedule before due. Any computed return earlier than the next learning day is offered on that next day. FSRS determines later intervals from its saved phrase state and actual review time. The three buttons, retention setting and daily eligibility rule are pragmatic defaults, not a claim of optimal language learning.

Keep FSRS JSON, due/first-study instants and a nullable `last_review` UTC instant on each phrase. On every explicit grade, compare `last_review` with today in the currently configured timezone. If it is null or another local date, call FSRS and update its JSON, due and `last_review` together, even for an early phrase-specific Practice grade. Another grade today records completed practice without calling FSRS. Due date controls automatic selection, not grading. First-study Next sets tomorrow's automatic recall without setting `last_review`; Skip never calls FSRS. Looking up a phrase, Show, Details, Hint, playback and recording do not grade or reschedule it.

## Daily allowance

Use the learner's timezone, initially Europe/Belgrade. Store `daily_due_limit` and `daily_new_limit` settings, defaulting to 20 due and five new, as independent categories permitting up to 25 completions. Validate nonnegative integers and reject both zero. Daily reads their current values on entry and each next selection rather than snapshotting them into the session. An increase can make more eligible phrases available after the previous limit was reached; there is no stored complete flag.

Count today's Daily and Extra completed recalls and first studies from indexed `practice_log` rows; a skipped recall consumes a due slot. Phrase-specific Practice consumes neither allowance. Opening or abandoning a phrase consumes none.

Keep two date-keyed Extra allowance targets. For each category with live configured limit `L` and today's completed count `C`, an explicit Extra action proposes `L` when `C < L`, otherwise `C + L`; store the maximum of that proposal and the existing target. Selection uses the larger of the live configured limit and stored target. A lower positive setting never shrinks a stored target. A zero category is disabled even with a prior target and receives no extension. A new learning date has no Extra target. Distinct presses without further completions do not compound the allowance; operation retries follow the [view and retry contract](#current-views-and-retries). These workload defaults are not research thresholds.

Automatic selection takes oldest due phrases first while due allowance remains, then new phrases while new allowance remains, one at a time. It does not cycle through not-due phrases. Skip leaves a due phrase due but excludes it from the current traversal; a later Extra action may offer it again.

## Completed practice

Use one `practice_log` row per completed first study, rated recall or Skip. No row is created by opening, changing cue, requesting Hint, revealing, recording, abandoning or Stop. Each row has a stable completion ID, nullable `phrase_id`, phrase wording and lesson snapshots, completion UTC instant and learning date, kind (first study or recall), nullable rating, Skip indicator, `fsrs_updated` and mode (Daily, Extra or Practice). Indexed date/category counts drive allowances and Today totals; distinct dates drive the streak and cumulative activity. Keep no separate historical attempt/review tables, duplicate daily totals, FSRS snapshots in the log or mastery score. Completed facts survive phrase deletion.

Saved takes retain their own cue/help/answer snapshots and stable take IDs without an attempt foreign key; a recording is useful even when no completion occurred.

## Current views and retries

One mutable current-session row keeps a traversal ID, learning date, entry origin (Daily or Extra), lesson filter, active phrase and phase, shown cues/hints and a bounded set of skipped phrase IDs. Every phrase appearance has a fresh view token distinct from the traversal ID, including ordinary advance, a repeated appearance of the same phrase and entry into Practice. Reveal, cue/hint and completion mutations bind to both that token and phrase identity. A stale token conflicts even when submitted with a new operation or completion ID.

**Daily** resumes the current regular view, including one opened by Extra; if none exists today it opens today's view. A new date closes an unfinished old view ungraded. **Extra** extends the date allowance and starts a fresh traversal, clearing skipped IDs; phrase selection follows separately and creates no completion. Opening phrase-specific **Practice** atomically closes any unfinished regular view ungraded and invalidates its token, even for the same phrase. It preserves the regular traversal's skipped IDs and date allowance. Practice is the sole valid view until it returns to details; subsequent Daily selects a fresh view instead of restoring the old one. Changing lesson affects the next new view, not the current phrase.

Each deliberate Extra press has a new stable operation ID; transport retries reuse it. One SQLite transaction persists its request payload, allowance changes, traversal replacement and result. An exact retry returns the persisted result without recalculating targets or replacing the current traversal, even after later completions, another tab's actions, a date change or restart. Changed payload under that ID conflicts. The client treats a replayed result as acknowledgement of that operation and reconciles the current view rather than restoring a superseded view.

A rating, Skip or first-study Next finalizes once and advances. One SQLite transaction validates the current phrase/view token, writes the log row, updates phrase FSRS JSON/due/`last_review` when eligible, and saves the next view with a fresh token and the operation's request/result. Resolve a persisted completion ID before checking current-view validity: an exact retry returns its original result even after later advances or restart; changed payload under that ID conflicts. A previously uncompleted stale view conflicts. Replayed responses cannot restore superseded client state. Failed saves leave the same card with an actionable retry using the same ID and no partial work. Rating and Skip advance immediately; only first-study **Next** completes ungraded study. There is no timed repair queue, dependency graph or repeated same-day promotion.

Each recall appearance chooses a random prepared cue, avoiding the phrase's last shown cue when another exists; retain that last cue on the phrase. **New cue** and requested **Hint** choose another prepared value when possible. The current view retains shown values and bounded cue/hint retry state for reload and exact retry; an exact operation retry returns its prior choice. Saved takes snapshot support seen at capture. No cue/hint event log is needed.

## Deletion

**Delete** in Library, phrase details or a revealed card back requires confirmation naming the phrase and the loss of its reference and saved recordings. Cancel changes nothing. Confirmed deletion invalidates its active view, removes phrase and takes, and blocks late uploads from recreating it. Deletion on a revealed back advances without a grade or practice-log row. Completed log rows keep their wording/lesson snapshots and a nullable phrase reference so past practice remains truthful. [Durable file cleanup](architecture.md#deletion-cleanup) handles interruption and unlink failure separately from the database transaction. A replacement phrase is a fresh import. There is no Suspend/Restore state. Never label a phrase fluent, automatic or mastered from a schedule or recording.

## Evidence behind the loop

- [Spaced practice and retrieval](research/spacing-and-grading.md) support returning on later days and retrieving after initial success. They do not select FSRS or an exact interval.
- [Study, feedback and help](research/phrase-prompts.md#study-first-when-the-phrase-is-unfamiliar) support learning unfamiliar wording before testing and correcting failed attempts. They do not require a multi-step hint ladder or timed repair queue.
- [Spoken production](research/phrase-prompts.md#match-practice-to-use) motivates saying the response. A silent self-check is permitted for convenience but is not evidence of spoken fluency.
- [Variable cues](research/phrase-prompts.md#vary-the-prompt) motivate rotation; the evidence does not establish spontaneous conversational transfer.
