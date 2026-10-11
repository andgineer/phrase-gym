# Decision: optional recording and automated comparison

2026-10-11. This supplements the [targeted-drilling decision](decision-targeted-drilling.md). The user explicitly requested recording/replay controls, a session-level option for times when they cannot speak, and automatic playback of their own version when recording is enabled. They endorsed the flow below and subsequently selected manual stopping and automatic database retention of all recordings.

## Agreed behaviour and endorsed working flow

Recording has a **session-level on/off switch**, accessible during practice as well as session setup. Switching it off allows the session to continue without recording. A microphone is not a prerequisite for using the trainer.

With recording on:

1. Show the situation or partner line together with the phrase-specific cue. The target has already been introduced.
2. The learner presses the **microphone-icon button** and says the complete phrase aloud. Give the icon-only control an accessible Start recording name.
3. While actively recording, the same button shows a **stop icon and “Stop recording”**. The learner taps it to finish. Manual stop is the agreed default and the only stopping method in the first version.
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

## Stopping the recording: manual by default; settings choice deferred

Accepted on 2026-10-11 after the user reviewed the discussion sketch:

> “this is in app settings (not implemented in 1st version), by default manual stop”

The first version uses **manual Stop recording**. Choosing between manual stop and automatic stop after a detected pause belongs in **app settings in a later version**. The stop-method setting and automatic stopping are outside first-version scope; manual remains the default when that setting is introduced. This decision concerns the stopping method, not whether recording itself defaults on or off.

| Option | User experience | Status |
|---|---|---|
| **Tap Stop recording** | The learner determines when the whole phrase is complete, including thinking pauses. | Accepted default; only stopping method in the first version. |
| Stop after a detected pause | Fewer taps, but hesitation can be mistaken for completion. | Later app-settings option; not implemented in the first version. Detection behaviour and timing remain unvalidated. |

This resolves the earlier open manual-versus-automatic stop choice. Preserve the session-level recording on/off switch as a separate, already agreed first-version control.

### Compact recording controls

Accepted UI refinement, 2026-10-11: remove “Say the whole phrase aloud.” and “Ready when you are.” from the drill. Use the microphone icon itself as the idle start control, with no visible Record label. During capture show the stop icon and **Stop recording** label. This replaces the previous Stop & compare wording; the automatic learner-take → model comparison still runs after stopping. The whole-phrase response remains the task even though its repeated instructional line is removed. Retain accessible control names and useful playback/error states without restoring idle filler text.

## Store all recordings; progress-comparison UI later

Accepted on 2026-10-11:

> “keep all the recordings in the DB … in future we could create nice UI to compare your progress (for the moment we do not need that - just store in DB)”

- **Automatically retain every recording in the app database from the first version**, including initial takes, imperfect attempts and rerecordings. Store the audio itself, not just a reference to temporary browser data. Rerecording adds a take; it never replaces the previous one.
- No Save recording action, opt-in baseline or manual selection is required. Do not automatically discard takes after comparison/session completion or expire them by age.
- Associate each take with its date/time, learning target, session/attempt, cue/content revision and support used, so future comparison can distinguish different practice conditions. Preserve audio format and capture status as well.
- Keep available audio from interrupted takes with an honest incomplete status. An empty or failed capture without audio remains a capture event, not a fabricated recording.
- **A historical progress-comparison interface is deferred.** The first version stores the recordings and retains the already agreed immediate self/model replay controls. A recordings timeline, baseline picker or progress-scoring feature is not required now.

This supersedes the assistant's proposal to retain only explicitly saved takes. Recording on/off defaults remain open. Automatic storage in the app database does not authorize sending recordings to an external AI service.

## Implementation implications to validate

These are engineering consequences and prototype checks, not additional user decisions:

- Detect actual recording/playback capabilities and test the ordered comparison on the user's phone. If playback is blocked or fails, show an explicit replay control rather than pretending it happened.
- Keep recording, processing, learner playback, model playback and self-check visibly distinct. Avoid multiple simultaneous players or recording the model playback into a learner take.
- Permission denial, empty/interrupted takes and unavailable hardware offer retry or recording-off practice without a memory-failure grade.
- Persist recording mode and actual response/support modality with the attempt; changing the session switch does not relabel earlier attempts.
- Persist audio bytes and their metadata in the database (BLOB storage in the proposed SQLite architecture). Queue pending audio locally until database persistence is acknowledged; retry without duplicate takes. A storage failure must remain visible and recoverable, not be reported as successful saving.
- Waveform/countdown presentation and remembering the last session's recording preference still need decisions. Historical comparison UI is deferred; no external AI upload or speech scoring follows from the recording requirement.

The exact screen presentation is in [interaction design](design/interaction-design.md). Include both modes and automatic comparison in P1; they are not optional polish to omit from the first design walkthrough.
