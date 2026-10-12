# Open problems and acceptance gaps

Updated **2026-10-12**. The user closed P2 engineering work while explicitly retaining the problems below. These are carried into later work; P2 closure does not establish device, speech-quality or release acceptance.

## AUDIO-01 — Mid-recording clipping

**Open; user-confirmed continuing failure.** After the local manual-test instructions, the user reported: “it continue clipping in the middle of recording”. The earlier reported setup was Safari on a MacBook Pro M1 with an external headset. The latest test page, Safari version, headset model/connection and whether the fault is in captured input, encoding or playback were not confirmed. Do not infer a root cause from this report.

The prototype requests echo cancellation, noise suppression and automatic gain control off, and uses optional play-and-record Audio Session support. The practice flow retains a microphone stream between takes; the audio probe releases it between takes. These measures have not resolved the reported issue. The synthetic continuity check uses three 2.1-second generated-tone takes per browser engine; it does not exercise the real headset or establish continuity of longer spoken recordings.

P2 has reference playback but no production microphone capture. Carry this defect into P3 capture implementation and P4 device acceptance. First distinguish microphone/input, encoder/container and playback failure with focused evidence on the user's actual setup. Keep recordings private and do not require a general review loop or repeated undirected manual tests. Close this problem only after the implemented capture path passes actual-device listening checks, including a longer continuous take, successive takes and pause/resume. Automated tests alone cannot close it.

The user requested stopping the current investigation and closing P2. No additional audio change or manual test is required for that closure.

## DEVICE-01 — iPhone and installed-app acceptance

**Untested.** A phone-accessible trusted HTTPS test origin and production capture/storage are still needed. P3/P4 must verify permission, recording/replay, interruption recovery, saved-take persistence and installed-app behavior before making the corresponding device claim.

## CONTENT-01 — Real reference speech and language quality

**Incomplete.** Actual actor/course clips and prepared Serbian reference speech/content need review. Synthetic tones establish container/byte/playback behavior, not complete wording, pronunciation, speech quality or linguistic correctness. Carry reference preparation and listening/content acceptance into the appropriate P3/P5 work; do not treat existing example fixtures as an accepted release pack.
