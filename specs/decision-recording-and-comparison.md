# Decision: optional recording and automated comparison

2026-10-11. This supplements the unchanged [targeted-drilling decision](decision-targeted-drilling.md). The user explicitly requested recording/replay controls, a session-level option for times when they cannot speak, and automatic playback of their own version when recording is enabled. They responded “great” to the proposed flow below and asked to preserve the session's solutions.

## Agreed behaviour and endorsed working flow

Recording has a **session-level on/off switch**, accessible during practice as well as session setup. Switching it off allows the session to continue without recording. A microphone is not a prerequisite for using the trainer.

With recording on:

1. Show the situation or partner line together with the phrase-specific cue. The target has already been introduced.
2. The learner presses **Record** and says the complete phrase aloud.
3. The recording ends. The working design uses **Stop & compare**; the exact stop mechanism remains open below.
4. Automatically play **the learner's recording first**, with the initial cue still visible.
5. Then reveal the complete phrase and play **the model**.
6. Offer the learner's self-check: **Got it**, **Try again**, or the different-wording/unclear-cue route.

The comparison screen retains **Play my recording**, **Play model**, and **Record another try**. The learner can interrupt playback. Rerecording belongs to the same target and practice history; it does not erase the first attempt or support already used.

This automation concerns the comparison sequence. Automatic recording start, automatic grading and automatic advancement to the next target were not requested or agreed. Playback is sequential, not two voices overlaid. The app must not reveal or play the answer before the attempted response, except when the learner explicitly requests model help.

With recording off:

1. Show the same useful context and phrase-specific cue.
2. Let the learner practise silently and press **Reveal & compare** when ready.
3. Reveal the complete phrase. Model audio plays only when requested.
4. Keep self-check and repair available without any microphone requirement.

Silent practice is not reported as spoken performance. Recording being off also cannot prove whether the learner spoke; explicit support for unrecorded spoken practice was not separately designed. Record only what the app observes or the learner explicitly reports.

## Stopping the recording: unresolved choice

| Option | User experience | Status |
|---|---|---|
| **Tap Stop & compare** | The learner determines when the whole phrase is complete, including thinking pauses. | Assistant's recommended starting point; not an explicit user selection. |
| Stop after a detected pause | Fewer taps, but hesitation can be mistaken for completion. | Alternative raised in discussion; not selected or validated. |

The last question asking the user to choose was answered with a general “great” and a request to save the decisions, not a specific choice. Preserve both the endorsed automation and this remaining uncertainty. Do not add silence detection as an assumed requirement.

## Implementation implications to validate

These are engineering consequences and prototype checks, not additional user decisions:

- Detect actual recording/playback capabilities and test the ordered comparison on the user's phone. If playback is blocked or fails, show an explicit replay control rather than pretending it happened.
- Keep recording, processing, learner playback, model playback and self-check visibly distinct. Avoid multiple simultaneous players or recording the model playback into a learner take.
- Permission denial, empty/interrupted takes and unavailable hardware offer retry or recording-off practice without a memory-failure grade.
- Persist recording mode and actual response/support modality with the attempt; changing the session switch does not relabel earlier attempts.
- Retention, explicit saving, upload, baseline recordings, waveform/countdown presentation and remembering the last session's preference still need decisions. No external AI upload or speech scoring follows from the recording requirement.

The exact screen presentation is in [interaction design](design/interaction-design.md). Include both modes and automatic comparison in P1; they are not optional polish to omit from the first design walkthrough.
