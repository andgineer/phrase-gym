# Open problems and acceptance gaps

Updated **2026-10-13**. The user closed P2 engineering work while explicitly retaining the problems below. These are carried into later work; P2 closure does not establish device, speech-quality or release acceptance.

## AUDIO-01 — Mid-recording clipping

**Latest production Safari/M1 check passes; broader device acceptance remains open.** On 2026-10-13, testing the production app locally with imported vault phrases, the user reported: “recording works fine, without clipping - in Safari macbook pro M1”. This is actual-device feedback, separate from synthetic tests. The report does not specify a headset, Safari version, take duration or pause/resume coverage.

The earlier prototype report was: “it continue clipping in the middle of recording”, with Safari on a MacBook Pro M1 and an external headset. The fault's capture, encoding or playback origin was not established. The latest successful production check supersedes the claim that clipping continues in the current tested setup; it does not establish a root cause or acceptance for the earlier external headset.

The prototype requests echo cancellation, noise suppression and automatic gain control off, and uses optional play-and-record Audio Session support. The practice flow retains a microphone stream between takes; the audio probe releases it between takes. Those prototype measures did not resolve the earlier reported issue. The synthetic continuity check uses three 2.1-second generated-tone takes per browser engine; it does not exercise the real headset or establish continuity of longer spoken recordings.

P3 implements production capture/storage and now has the successful local listening report above. P4 retains broader device acceptance, including longer continuous takes, successive takes and pause/resume. If clipping recurs, distinguish microphone/input, encoder/container and playback failure with focused evidence on the affected setup. Keep recordings private and do not require a general review loop or repeated undirected manual tests. Automated tests alone cannot establish actual-device acceptance.

The user requested stopping the current investigation and closing P2. No additional audio change or manual test is required for that closure.

## DEVICE-01 — iPhone and installed-app acceptance

**Untested.** Production capture/storage is implemented in P3. A phone-accessible trusted HTTPS test origin and actual P4 device checks are still needed to verify permission, recording/replay, interruption recovery, saved-take persistence and installed-app behavior before making the corresponding device claim.

## CONTENT-01 — Real reference speech and language quality

**Incomplete.** Actual actor/course clips and prepared Serbian reference speech/content need review. Synthetic tones establish container/byte/playback behavior, not complete wording, pronunciation, speech quality or linguistic correctness. Carry reference preparation and listening/content acceptance into the appropriate P3/P5 work; do not treat existing example fixtures as an accepted release pack.
