# Learning loop and observable evidence

The objective is easier access to useful language when speaking. This MVP loop measures practice and self-reported recall, not spontaneous conversation, pronunciation accuracy or mastery. Later [Extra practice and adult engagement](decision-transfer-and-engagement.md) build on it while keeping their outcomes separate from memory grades. [Research and its limits](research/README.md) distinguish empirical findings from the defaults below.

## First meeting

New cards start with the complete phrase, a comprehensible meaning/use note and playable reference. Invite the learner to say it once; recording is optional. **Next** completes study without a grade or an invented successful recall. Importing a course phrase does not establish that the learner already knows it. The MVP uses this same short first meeting for every new card rather than inferring familiarity.

First independent recall is offered on the next learning day. This is a simple spacing choice, not a research-derived optimum. Completing study today must not immediately reopen the same card as an independent test. Short sessions finish without a wait screen.

## Independent recall

1. Show a short situation and, when useful, the other person's preceding line. Hide the answer, reference audio, answer-bearing help and lesson/item titles that give it away.
2. Invite a spoken response, with an optional recording. Silent self-check remains available without microphone access. There is no countdown, speed score or required mock dialogue.
3. **Show** reveals the reference wording and short explanation; reference playback and comparison with the learner's take become available. **Details** also reveals the answer and opens the longer explanation. The learner rates the response made before help or reveal.
4. **Not yet** means missing/wrong wording or inability to retrieve without help; **Effort** means correct recall with difficulty; **Got it** means comfortable correct recall. A sensible alternative is not a language error: use the card's accepted alternatives, or suspend an ambiguous card from its detail view.
5. The learner may rehearse with the back-side microphone after reveal and before rating. Pressing **Not yet**, **Effort** or **Got it** saves the rating and advances automatically; no second response or **Next** is required. Repetition, replay and a new take cannot replace the original response or produce another schedule update.

**Skip** closes the attempt without a rating and advances automatically. Revealing without an attempted response, inspecting an explanation, or a microphone/technical failure does not automatically mean forgetting. Requested Hint or Details does not hide rating controls; the learner can report what happened before using help. No AI judges speech or assigns a grade.

## Scheduling choices

Use the Python FSRS library with population parameters, desired retention 0.90, no short learning/relearning steps and no fuzzing. Map Not yet / Effort / Got it to Again / Hard / Good. Pin the selected library version and test integration; do not maintain a handwritten numerical port or require compatibility with prototype arithmetic fixtures.

At most one independent scheduled review per card per learning day changes its schedule. The first check follows completed study; subsequent checks require the card to be due. Any computed return earlier than the next learning day is offered on that next day. FSRS determines later intervals from actual review time/history. The three buttons, retention setting and daily eligibility rule are pragmatic defaults, not a claim of optimal language learning.

Use the learner's timezone, initially Europe/Belgrade. Daily has at most 20 completed cards and five completed first meetings per learning day across all lessons; Extra and phrase-detail Practice do not consume that budget. Prefer oldest due cards, then new cards. An opened or abandoned card consumes no budget. A small date-keyed Daily counter records completed cards and first meetings, updated once in the same transaction as finalizing a Daily attempt; Extra and phrase-detail Practice never increment it. These are workload defaults, not research thresholds. The **Daily** button opens eligible work or continues its unfinished session; an empty queue says when work returns. **Extra** uses the same card path without a session limit and supersedes the MVP's single-card **Practise freely** action.

Select a prepared cue randomly for each appearance, avoiding the immediately previous cue when another is available. Looking up a card, opening Details, showing the answer, requesting a hint, playing audio or recording does not grade it or move its due date. Only an explicit rating can update FSRS when the card is due and has not already had a scheduled review that learning day. Preparation avoids prompts that give away another card's answer in the same source material.

There is no timed repair queue, five-minute/two-other-card gate, introduction dependency graph, per-profile policy versioning or repeated same-day promotion. Skipping a due card leaves its due date unchanged and records no grade. Daily does not reoffer it on the same learning day; Extra postpones it behind other available phrases before cycling.

## Honest history and card problems

Save each finalized review once, with date, cue, original response mode, whether help preceded it, the self-rating and resulting due date. Reload/restart retains an unfinished Daily card and saved work. Returning to entry or opening Extra pauses that Daily card; Daily resumes it through the same button, except when Extra or phrase-detail Practice selects that same phrase: then close the paused Daily attempt ungraded before opening the other mode, without a Daily counter increment. Closing an unfinished Extra or phrase-detail Practice card records no grade. Interrupted or duplicate submissions never fabricate grades. Rating is atomic with any schedule update, then the client advances. Skip advances without a grade. Only first study uses **Next** to complete its ungraded meeting. A failed rating save stays on the back with an actionable retry using the same operation ID; duplicate presses cannot create another grade. The server rejects stale or conflicting practice-state mutations and returns the original result for an exact retried operation. Distinct takes may still upload to a finalized or ungraded-closed attempt without changing its grade, schedule or Daily counters. Completion, budget and streak use the learning day in the configured timezone at completion, including for an attempt crossing midnight; timezone changes do not rewrite history or permit a duplicate scheduled review. Keep recordings separately from grading: a take can be useful even when its attempt is ungraded.

**Suspend** on the phrase detail view removes a defective card from practice without marking a failure; **Restore** returns it. In the MVP, replace defective wording by importing a corrected card and leaving the old card suspended with its history; no automated recreation system is needed. Restoring a card does not replay old ratings. Changes to the target or prompt during an attempt invalidate that attempt's grade.

Show due/new counts and dated review history. Supported study, free rehearsal and later independent recall remain visibly different. Never label a card fluent, automatic or mastered from its schedule or recording.

## Evidence behind the loop

- [Spaced practice and retrieval](research/spacing-and-grading.md) support returning on later days and retrieving after initial success. They do not select FSRS or an exact interval.
- [Study, feedback and help](research/phrase-prompts.md#study-first-when-the-phrase-is-unfamiliar) support learning unfamiliar wording before testing and correcting failed attempts. They do not require a multi-step hint ladder or timed repair queue.
- [Spoken production](research/phrase-prompts.md#match-practice-to-use) motivates saying the response. A silent self-check is permitted for convenience but is not evidence of spoken fluency.
- [Variable cues](research/phrase-prompts.md#vary-the-prompt) motivate rotation; the evidence does not establish spontaneous conversational transfer.
