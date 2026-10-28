# Phrase practice and adult engagement

The product goal is to expand the language an adult actually chooses while speaking, including useful expressions encountered in courses and videos. Independent card recall is an intermediate outcome. The app complements listening, reading and real conversation; richer language means appropriate expressive choices, not a quota of impressive phrases.

[Research on conversational uptake](research/automatic-use.md) supports deliberate phrase instruction combined with speaking practice, including gains in unscripted classroom conversations. It does not establish durable everyday transfer for this app or isolate one necessary exercise. The [delivery ladder](plan/implementation-plan.md) next extends phrase recall with optional unlimited practice. That extension is not an unscripted speaking activity.

## Daily and Extra phrase recall

Use personally useful expressions with their meaning, communicative purpose, register and source. The first MVP represents complete examples. Daily and Extra use the same phrase card, concise situation cues, optional requested recall hints, answer reveal and recording/replay.

Daily contains the scheduled cards waiting today and retains its session bounds. Extra starts directly or after Daily and continues through the available library without a session cap until the learner stops. It offers each phrase practised in Daily today once first, then recently practised phrases, then due and other available cards; after one traversal it cycles again. The same card's schedule can change at most once per learning day. Hints, answer exposure and not-yet-due attempts are supported practice, not independent reviews. New cards studied in Extra first return for independent recall on the next learning day.

For **So far, so good.**, one cue is **Colleague asks: How's it going?** and another is **Friend asks: How's the new job?** The learner says the short phrase, can request one of its brief hints, then sees the phrase and short explanation on the card back. The full phrase is enough; the activity does not demand extra speech.

This is phrase retrieval in varied contexts. Reusing the cue and requesting a hint can help practise recall, but neither demonstrates freer speaking or natural use outside the app. A distinct freer-speaking activity needs its own functional design before scheduling it. No AI conversation partner, countdown, memorized whole script or compulsory target insertion is required.

## Evidence and history

Record independent recall and supported practice distinctly. Record the cue and assistance so a recording's circumstances remain understandable.

Only eligible independent phrase recall updates FSRS, whether it occurs in Daily or Extra. An attempt is eligible only when its phrase is due, its wording and hint have not been shown and the learner can rate an actual attempt to retrieve that phrase. Supported responses, not-yet-due practice, repeated same-day attempts and mission completion never create or upgrade memory grades. Seeing a phrase in Extra follows the same supported-exposure rule as study or lookup before a later card review. Another natural response to a broad cue is not automatically a failure of the target phrase.

Do not label expressions automatic, fluent or mastered from card intervals, phrase counts, completed missions or recordings. Software checks verify these distinctions and saved state; they do not establish learning effectiveness. No mandatory learner experiment is a delivery gate.

## Adult engagement

Adults are the intended audience. [Motivation research](research/motivation.md) supports designing for choice, a sense of progress and meaningful activity, while separating enjoyment from persistence and learning. Engagement is planned product work, not an assumption that adult discipline makes repetitive drills sustainable.

- Offer chosen missions using useful phrase practice with a clear finish and varied cues. Do not label repeated card answers as freer speech.
- Show concrete progress: dated unaided recall, supported practice and comparable recordings, each with its own label. Avoid a single invented fluency score.
- Let the learner choose a modest weekly target, pause and return without debt, penalties or losing accumulated progress. Short sessions have a clear endpoint.
- Offer optional points or achievements for completing useful practice, including honest unsuccessful attempts. Rewards never depend on choosing a favorable self-rating, repeating easy cards indefinitely or forcing a phrase into unsuitable speech. Turning rewards off leaves learning available.

Weekly targets and rewards are defaults to refine from actual use, not scientifically optimal prescriptions. There is no required public leaderboard, competition, child-oriented presentation, elaborate virtual economy or punitive daily streak.
