# Decision: one practice card — 2026-10-12

The latest manual feedback supersedes earlier P1 proposals for separate cue/check/practice screens, a recording-mode switch, supported-only Continue controls, Practice again, and a detached Learning materials section. P1 is still a disposable prototype; this does not approve the production phase.

## Accepted interaction

- Keep one practice card with the phrase, recording/replay, reference audio and three self-check buttons. Recording is optional on every attempt. Remove Recording: On/Off entirely; learners can reveal and rate without granting microphone permission.
- After recording or pressing Show answer, replace the gaps with the full phrase on the same card. Keep the controls in place. Remove Practice again: the microphone is directly available for another take.
- Use one round audio control: microphone at rest, a stop icon during capture, a stop icon during playback. Stop playback restores the microphone. Do not show a separate idle Stop playback button. Preserve accessible action names and distinguish the recording and playback states visually.
- Keep Show answer and Show & explain together on the card, with different visual treatments. The current prototype uses an outlined Show answer button and a blue book-icon link for Show & explain. Explanation expands inside the same card and uses the native language.
- Always offer the three self-check choices. Use icons and distinct red/amber/green treatments, retaining text labels. Disable grading only while microphone permission/capture is active; grading can stop playback and advance.
- Remove Other answer / problem, I couldn’t recall it, and Continue without a recall grade. These controls must not return through an older plan or prototype sketch.

## Evidence and storage

Showing the answer snapshots the learner's original self-reported recall before revealing it. Recording snapshots the original response before capture. Direct grading without a recording records self-reported memory evidence; it does not claim spoken performance. A later take must not overwrite the original response's metadata. Every take is retained, and replay uses the latest take.

Ratings stay available when learning materials or a previous lookup supplied the answer. Store the chosen rating together with the support metadata; do not silently convert it into an independent-recall success or promote the schedule on that evidence. Supported results retain a later retrieval opportunity. This replaces the confusing UI restriction, not the distinction between assisted practice and recall. Actual scheduling remains a later phase.

## Reported Safari audio problem

The user reports recurring mid-recording clipping on Safari, MacBook Pro M1, with an external headset, while other applications work normally after at most their first take. The cause is not confirmed by this report alone.

The prototype previously reacquired and closed the microphone for each take. Version 0.1.4 keeps one microphone stream ready after the first explicit recording request while practice remains active, and uses the optional Audio Session API's play-and-record type when available. Only explicit takes are recorded/stored. The user subsequently removed the idle microphone-ready note. Pause/navigation, backgrounding, session completion and leaving the page release the microphone; a cancelled permission request releases any late stream. Audio-only self-check never requests it.

Request echo cancellation, noise suppression and automatic gain control off, and retain non-identifying actual capture settings with each take. Let the recorder flush its final data before releasing resources. This is a mitigation for repeated device/audio-route initialization and unwanted signal processing, not a claim that Safari/headset clipping has been reproduced or fixed.

References: [Audio Session specification](https://www.w3.org/TR/audio-session/) and [Media Capture and Streams](https://www.w3.org/TR/mediacapture-streams/). Repeated synthetic recordings can verify stream reuse, storage and encoded continuity; the user's actual Safari/headset listening check remains necessary.

## Verification

Version 0.1.4 checks: **12 core scenarios, 9 browser walkthroughs, 16 storage/audio regressions and 2 audio-continuity runs**, all passing. The UI walkthrough covers always-available self-check, no microphone request for silent use, inline explanation, colored/icon ratings, reusable record/stop controls, repeated takes, original-response preservation, 320px/200% text, language navigation, reload and offline playback. Cue and expanded-explanation screenshots were inspected.

Chromium and WebKit each encoded and stored three generated-tone takes with one stream acquisition, without measured middle-of-take gaps; cancel ended the stream. WebKit required a disposable persistent test profile because its ephemeral profile rejected IndexedDB Blobs. Actual Safari with the user's external headset remains unverified. These checks do not constitute a new fresh-session review loop.

## Recording hint and rating confirmation — version 0.1.5

Show **Press again to stop recording** beside the audio controls only while actively recording; hide it immediately when capture stops, including during processing/playback. Do not show the former Microphone ready until you pause label. Localize the instruction in all three UI languages.

Rename **Original result retained** to **Previous rating saved**. This banner confirms the self-check rating for the preceding phrase and accompanies Undo; it is not a message about the current recording.
