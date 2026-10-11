# Phrase Gym — interaction design discussed in this session

Updated 2026-10-11. This is a written design for review and future prototyping, not an implemented or visually approved app. The [drilling decision](../decision-targeted-drilling.md) and [recording decision](../decision-recording-and-comparison.md) define agreed behaviour. The final visual style and complete navigation remain open.

## Main practice screen

Recommended layout: one expression at a time, situation/partner line above a prominent phrase cue, a clear full-phrase instruction, one primary response action and visible help controls. Keep the situation with the cue so the learner knows both what they are doing and which learned expression is intended.

Illustrative recording-on screen for a known expression:

```text
‹ Pause                         3 of 8
Recording: On

A colleague's day
You meet a colleague during a break.
Ask how their day has been up to now.

        […] your day been so far?

Say the whole phrase aloud.

             Record

       More help    Show phrase
```

The learner says “How has your day been so far?” in full. The missing chunk is not the whole response. The sample count, labels and screen spacing are illustrative, not final design decisions. Each real cue requires its own review against competing completions.

## Recording-on interaction

| Step | What the learner sees/hears | What the learner does |
|---|---|---|
| Cue | Situation/partner line, identifying phrase chunks, Record control. | Read/hear the cue and press Record. |
| Recording | The same cue, a clear recording state, and a way to finish. | Say the whole phrase. The written working flow uses Stop & compare. |
| Own version | Automatic playback of the just-recorded take; keep the initial cue visible. | Listen; interrupt playback if desired. |
| Model | Reveal the complete phrase and automatically play its model audio after the learner take. | Compare the two versions. |
| Self-check | Got it, Try again, Different wording; replay controls remain available. | Report the result or practise another take. |
| Continue | Advance explicitly to the next item or appropriate repair/return. | Continue when ready; automatic advancement was not agreed. |

Comparison controls:

```text
     How has your day been so far?

      ▶ My recording    ▶ Model
         ↻ Record another try

     Got it → Next phrase
     Try again    Different wording
```

Recording is optional for the session and the switch is accessible during practice. Recording start is an explicit learner action in this proposal. The choice between manual Stop & compare and automatic stop after a pause is still open; manual stop is the recommended initial design. Do not silently turn “automation” into automatic grading or a permanent open microphone.

## Recording-off / cannot-speak interaction

Show the same phrase-specific task and use **Reveal & compare** as the primary action. The learner can recall silently, reveal the full phrase and request model audio manually. No microphone permission is needed. Keep help, repair and alternative/unclear-cue reporting available. Describe the result as silent practice rather than spoken performance.

The session switch controls recording; a future explicit unrecorded-spoken mode would require clear wording and evidence handling. Do not claim that disabling a microphone proves whether the learner spoke.

## New expressions and model help

For a new target, first show/hear the full expression, its meaning/use and a useful example or short dialogue. The learner studies or repeats it before a recall cue appears. Model exposure and immediate repetition are learning practice, not independent recall.

During recall, More help reveals additional meaningful material; Show phrase reveals the complete model. The necessary distinguishing cue belongs in the initial prompt. Record all extra help, including any audio that exposes hidden material. After repair, revisit the target after intervening material where possible. A second successful take does not erase the first difficulty.

## Different wording and unclear cues

The proposed compact outcome set is **Got it**, **Try again**, **Different wording**. Different wording opens a small choice between “I chose another expression” and “The cue didn't make the intended expression clear.” Exact labels can be refined in the walkthrough.

Good alternative English is not automatically a memory failure. An unclear cue needs correction. For fixed phrases, show explicitly allowed equivalents; for words/frames, show which parts may vary. Keep one target/result history across changed masks and recordings.

## Specific dialogue rehearsal

Learn the complete short exchange first. Rehearse the learner's side: partner line → target-specific cue → full response → comparison. Leave supporting lines readable. Recording-on turns follow the same own-take/model sequence; recording-off turns remain usable silently. Isolate a difficult target turn, repair it, then reinsert it at its original point without creating another learning target or schedule.

## Visual direction and remaining design work

The assistant recommended a calm, focused style: large phrase text, warm neutral background, one restrained accent and generous spacing. A more energetic gym-like style with stronger colours/progress was offered as an alternative. **The user did not select either visual style.** No palette, typography, navigation, home screen, progress dashboard, carousel choice or earlier HTML sketch is accepted by implication.

Before implementation of unresolved screens, continue the design discussion on those choices. P1 should let the user inspect the concrete seven practice scenarios, both recording modes, automatic comparison, permission/failure states, pause/resume and honest results. The deliverable is a reviewed prototype and actual-device evidence, not an assumed visual approval.
