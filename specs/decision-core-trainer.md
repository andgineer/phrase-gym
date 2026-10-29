# Learning loop and observable evidence

The objective is easier access to useful language when speaking. This MVP loop measures practice and self-reported recall, not spontaneous conversation, pronunciation accuracy or mastery. Later [Extra practice and adult engagement](decision-transfer-and-engagement.md) build on it while keeping their outcomes separate from memory grades. [Research and its limits](research/README.md) distinguish empirical findings from the defaults below.

## First meeting

New phrases start with the complete phrase, a comprehensible meaning/use note and playable reference. Invite the learner to say it once; recording is optional. **Next** completes study without a grade or an invented successful recall. Importing a course phrase does not establish that the learner already knows it. The MVP uses this same short first meeting for every new phrase rather than inferring familiarity.

First independent recall is offered on the next learning day. This is a simple spacing choice, not a research-derived optimum. Completing study today must not immediately reopen the same phrase as an independent test. Short sessions finish without a wait screen.

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

Use the learner's timezone, initially Europe/Belgrade. Daily and Extra use the same due-first, then new selector and independent per-session due/new limits stored in settings, defaulting to 20 due and five new. Snapshot the limits when a session starts. Either limit may be zero, but both cannot be zero. The learner can start another bounded session; every explicit **Extra** press starts a fresh one. Extra starts with zero session counters; phrases are chosen one at a time as the learner advances. Daily statistics remain cumulative across sessions. **Daily** opens the current session, including one started by Extra; it creates the first session of a learning day only when none exists. A completed session remains complete until Extra is pressed or the next learning day starts. These workload defaults are not research thresholds. The MVP's single-card **Practise freely** action gives way to phrase-specific **Practice** from details.

Select a prepared cue randomly for each appearance, avoiding the immediately previous cue when another is available. Looking up a phrase, opening Details, showing the answer, requesting a hint, playing audio or recording does not grade it or move its due date. On an explicit rating, call FSRS whenever `last_review` is not today in the configured timezone, even if the phrase is not due. Due dates only filter automatic selection. Preparation avoids prompts that give away another phrase's answer in the same source material.

There is no timed repair queue, five-minute/two-other-card gate, introduction dependency graph, per-profile policy versioning or same-day promotion. A skipped due phrase remains due but is not reoffered within that session; a new session may offer it again. Automatic selection ends after the due/new limits or when no eligible phrase remains. It does not cycle through not-due phrases.

## Phrase state, honest progress and deletion

Keep FSRS JSON and due/first-study instants on the phrase, plus a nullable `last_review` UTC instant set only when an explicit grade calls FSRS. Compare it with today in the currently configured timezone; when it is null or another local date, every explicit grade calls FSRS and updates JSON, due and `last_review` together. A repeated grade today updates practice statistics only. First-study Next sets tomorrow's automatic recall without setting `last_review`, so a learner-initiated rated Practice of that phrase later today can call FSRS once. Show, Details, Hint, search, audio, recording and Skip do not reschedule.

A small mutable current session stores its mode, lesson, phrase-view phase/support, due/new counters and a bounded operation receipt. It supports reload, exact retry and stale-tab conflicts; it is not a historical attempt or review log. Completing a phrase atomically updates schedule/marker when eligible, session counters and date-keyed practice aggregates. New Extra sessions keep those aggregates. Rating and Skip advance immediately; first study alone uses **Next**. Failed saves keep the same card view for retry with the same operation ID. Recording uploads use independent stable IDs and may arrive after leaving the card view without changing grades or counts. Saved takes keep their own cue/help/answer snapshots so later playback remains understandable. With no attempt or review tables, show current due state, saved recordings and aggregate practice facts rather than a detailed rating timeline. Never label a phrase fluent, automatic or mastered from a schedule or recording.

**Delete** in Library, phrase details or a revealed phrase back requires confirmation naming the phrase and explaining that the phrase, reference and saved recordings will be removed. Cancel changes nothing. Confirmed deletion invalidates an open phrase view; pending uploads cannot restore the phrase or leave orphan files. Completed practice-day aggregates remain labeled as past activity. Replacing defective wording means importing a new phrase after deleting the old one. There is no Suspend/Restore state.

## Evidence behind the loop

- [Spaced practice and retrieval](research/spacing-and-grading.md) support returning on later days and retrieving after initial success. They do not select FSRS or an exact interval.
- [Study, feedback and help](research/phrase-prompts.md#study-first-when-the-phrase-is-unfamiliar) support learning unfamiliar wording before testing and correcting failed attempts. They do not require a multi-step hint ladder or timed repair queue.
- [Spoken production](research/phrase-prompts.md#match-practice-to-use) motivates saying the response. A silent self-check is permitted for convenience but is not evidence of spoken fluency.
- [Variable cues](research/phrase-prompts.md#vary-the-prompt) motivate rotation; the evidence does not establish spontaneous conversational transfer.
