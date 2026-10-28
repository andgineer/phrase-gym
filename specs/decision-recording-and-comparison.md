# Recording, comparison and saved replay

Recording and reference comparison are core product functions. Preserve the working Vue audio implementation and its browser-specific handling while replacing the backend.

## Observable behavior

- Recording is optional in study, independent recall and free rehearsal. Silent self-check never asks for microphone access. Start and stop are explicit learner actions; no voice detector or automatic speech grade is required.
- After stopping, retain the final encoder output durably in the browser before navigation; upload continues independently of card navigation or rating, with pending/error recovery. On a hidden-answer front, automatic playback may play the learner take alone. Once the answer is revealed, comparison plays the complete learner take followed by the complete reference; either can also be replayed separately. In study/rehearsal the answer is already visible. Only one playback runs at a time.
- Replaying or recording again cannot change which response the learner is grading. Every take remains a separate recording, including imperfect or supported repetitions. There is no Save button and no automatic expiry.
- Show pending/save failure truthfully. Reload/restart preserves acknowledged takes and their card/attempt context; retrying a pending save does not duplicate it. Earlier takes are never overwritten by a new one.
- A saved take is accessible again from its card/history after reload and application restart. Keep this a simple replay control, not an audio analysis dashboard.
- An outside pause, such as a headset media control, ends automatic comparison quietly and leaves both replays available. Navigation/cancellation stops media and releases its resources. Microphone failure offers retry or silent practice and creates no forgetting grade.

Learner recordings stay private and never enter AI preparation or speech providers. Store original bytes and emitted MIME without server-side transcoding or speech inspection. Retain existing upload bounds and pending browser storage where needed for the working recorder. Warn about the 8 MiB capture cutoff before the 12 MiB upload limit; this is a technical size limit, not a desired speaking duration. Preserve recoverable interrupted audio, but do not claim to recover an unflushed tail after abrupt browser termination.

## Browser constraints

These are observed device constraints worth retaining, not a reason to rewrite audio:

- **iPhone/touch:** native microphone defaults; acquire a fresh stream per take and release it after final recorder data, before comparison playback. Forcing processing off previously produced silent captures.
- **macOS Safari/desktop:** request processing off and keep input open across takes until leaving practice. Reopening per take switched Bluetooth profiles and interrupted playback; do not apply the touch-device rule to desktop.
- Use the emitted MIME, even when it appears only after recording starts. Keep playback cancellation bound to its initiating take/card so late events cannot play or save against another card.
- Feature-detect the optional [Audio Session API](https://www.w3.org/TR/audio-session/). Capture uses auto or play-and-record; playback-only while input is live can end tracks. Use playback for comparison after input release where appropriate. Browsers without a writable API use native behavior.

The [WebKit capture-interruption notes](https://webkit.org/blog/18194/release-notes-for-safari-technology-preview-251/) support caution about disabling echo cancellation; actual-device checks remain decisive for this app. Desktop input retention is released on leaving practice, not after every take.

## Acceptance boundary

Automated browser tests must be silent and cover final encoder data, ownership, retry and saved replay against the real backend. They do not certify audible microphone capture or headset routing. The learner checks variable-length speech through its final words, comparison and saved replay on the MacBook used for the MVP. Retain iPhone fixes, but new phone access/setup and headset/platform qualification do not block the local desktop MVP.

Playback comparison is a learner aid, not a validated automatic pronunciation assessment or proof of conversational transfer. Backups include every saved take under the [stopped-app procedure](architecture.md#backup-and-restore).
