# Learning loop and observable evidence

The objective is easier access to useful language when speaking. The application measures practice and self-reported recall, not spontaneous conversation, pronunciation accuracy or mastery. [Research and its limits](research/README.md) distinguish empirical findings from the defaults below.

## First meeting

New cards start with the complete phrase, a comprehensible meaning/use note and playable reference. Invite the learner to say it once; recording is optional. **Next** completes study without a grade or an invented successful recall. Importing a course phrase does not establish that the learner already knows it. The MVP uses this same short first meeting for every new card rather than inferring familiarity.

First independent recall is offered on the next learning day. This is a simple spacing choice, not a research-derived optimum. Completing study today must not immediately reopen the same card as an independent test. Short sessions finish without a wait screen.

## Independent recall

1. Show a short situation and, when useful, the other person's preceding line. Hide the answer, reference audio, answer-bearing help and lesson/item titles that give it away.
2. Invite a spoken response, with an optional recording. Silent self-check remains available without microphone access. There is no countdown, speed score or required mock dialogue.
3. **Reveal** shows the reference wording and short explanation; reference playback and comparison with the learner's take become available. The learner rates the response made before reveal.
4. **Not yet** means missing/wrong wording or inability to retrieve without help; **With effort** means correct unaided recall with difficulty; **Recalled** means correct unaided recall comfortably. A sensible alternative is not a language error: use the card's accepted alternatives, or **Unclear card** to exclude an ambiguous prompt from grading.
5. After Not yet or With effort, invite one supported repetition, then **Next**. After Recalled, advance. Repetition, replay and a new take cannot replace the original response or produce another schedule update.

The learner may instead choose **Study / skip grading** before answering. Revealing without an attempted response, inspecting an explanation, or a microphone/technical failure does not automatically mean forgetting. No AI judges speech or assigns a grade.

## Scheduling choices

Use the Python FSRS library with population parameters, desired retention 0.90, no short learning/relearning steps and no fuzzing. Map Not yet / With effort / Recalled to Again / Hard / Good. Pin the selected library version and test integration; do not maintain a handwritten numerical port or require compatibility with prototype arithmetic fixtures.

At most one independent scheduled review per card per learning day changes its schedule. The first check follows completed study; subsequent checks require the card to be due. Any computed return earlier than the next learning day is offered on that next day. FSRS determines later intervals from actual review time/history. The three buttons, retention setting and daily eligibility rule are pragmatic defaults, not a claim of optimal language learning.

Use the learner's timezone, initially Europe/Belgrade. Prefer oldest due cards, then introduce up to five new cards within a session of at most 20 cards. These are workload defaults, not research thresholds. **Resume** continues unfinished work; **Start** offers eligible work; an empty queue says when work returns. An explicit **Practise freely** action permits ungraded rehearsal at any time without changing due obligations.

Rotate available cues in simple round-robin order across independent reviews. A card shown with its answer in study, lookup or free practice is supported for the rest of that learning day; it cannot become a fresh independent test by reopening the app. Preserve eligibility of the original response captured before its own reveal. Keep this as ordinary per-card attempt/exposure state, not a general cross-card exposure engine. Preparation avoids prompts that give away another card's answer in the same batch.

There is no timed repair queue, five-minute/two-other-card gate, introduction dependency graph, per-profile policy versioning or repeated same-day promotion. Existing due work skipped for study remains due but is not offered as independent recall again that day.

## Honest history and card problems

Save each finalized review once, with date, cue, original response mode, whether help preceded it, the self-rating and resulting due date. Reload/restart retains the last confirmed session position and saved work. Interrupted or duplicate submissions never fabricate grades. Keep recordings separately from grading: a take can be useful even when its attempt is ungraded.

**Unclear card** or **Suspend** immediately removes a defective card from practice without marking a failure. The library exposes suspended cards and **Restore**. In the MVP, replace defective wording by importing a corrected card and leaving the old card suspended with its history; no automated recreation system is needed. Restoring a card does not replay old ratings. Changes to the target or prompt during an attempt invalidate that attempt's grade.

Show due/new counts and dated review history. Supported study, free rehearsal and later independent recall remain visibly different. Never label a card fluent, automatic or mastered from its schedule or recording. Optional reports of real-life use, if added later, remain separate from memory grades.

## Evidence behind the loop

- [Spaced practice and retrieval](research/spacing-and-grading.md) support returning on later days and retrieving after initial success. They do not select FSRS or an exact interval.
- [Study, feedback and help](research/phrase-prompts.md#study-first-when-the-phrase-is-unfamiliar) support learning unfamiliar wording before testing and correcting failed attempts. They do not require a multi-step hint ladder or timed repair queue.
- [Spoken production](research/phrase-prompts.md#match-practice-to-use) motivates saying the response. A silent self-check is permitted for convenience but is not evidence of spoken fluency.
- [Variable cues](research/phrase-prompts.md#vary-the-prompt) motivate rotation; the evidence does not establish spontaneous conversational transfer.
