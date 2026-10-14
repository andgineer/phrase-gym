# Open problems and acceptance gaps

Updated **2026-10-14**. The user closed P2 engineering work while explicitly retaining the problems below. These are carried into later work; P2 closure does not establish device, speech-quality or release acceptance.

## AUDIO-01 — Mid-recording clipping

**Latest production Safari/M1 check passes; broader device acceptance remains open.** On 2026-10-13, testing the production app locally with imported vault phrases, the user reported: “recording works fine, without clipping - in Safari macbook pro M1”. This is actual-device feedback, separate from synthetic tests. The report does not specify a headset, Safari version, take duration or pause/resume coverage.

The earlier prototype report was: “it continue clipping in the middle of recording”, with Safari on a MacBook Pro M1 and an external headset. The fault's capture, encoding or playback origin was not established. The latest successful production check supersedes the claim that clipping continues in the current tested setup; it does not establish a root cause or acceptance for the earlier external headset.

The prototype requests echo cancellation, noise suppression and automatic gain control off, and uses optional play-and-record Audio Session support. The practice flow retains a microphone stream between takes; the audio probe releases it between takes. Those prototype measures did not resolve the earlier reported issue. The synthetic continuity check uses three 2.1-second generated-tone takes per browser engine; it does not exercise the real headset or establish continuity of longer spoken recordings.

P3 implements production capture/storage and now has the successful local listening report above. P4 retains broader device acceptance, including longer continuous takes, successive takes and pause/resume. If clipping recurs, distinguish microphone/input, encoder/container and playback failure with focused evidence on the affected setup. Keep recordings private and do not require a general review loop or repeated undirected manual tests. Automated tests alone cannot establish actual-device acceptance.

The user requested stopping the current investigation and closing P2. No additional audio change or manual test is required for that closure.

## DEVICE-01 — iPhone acceptance

**Upload rejection reported; corrected retention awaits an actual-device retest.** On 2026-10-14 the user reached the private Tailscale HTTPS origin on iPhone, acknowledged session events and played recordings locally, but four complete takes failed server decoding validation with HTTP 422. The [learner recording storage decision](decision-learner-recording-retention.md) removes server decoding and container inspection; pending takes can retry unchanged after server restart. The user reports the latest iOS, without a numeric version. Saved-take replay/reopening, interruptions, headset behavior and VoiceOver remain acceptance gaps. Offline installation is withdrawn by the [online-only decision](decision-online-only.md).

## CONTENT-01 — Real reference speech and language quality

**Incomplete.** Actual actor/course clips and prepared Serbian reference speech/content need review. Synthetic tones establish container/byte/playback behavior, not complete wording, pronunciation, speech quality or linguistic correctness. Carry reference preparation and listening/content acceptance into the appropriate P3/P5 work; do not treat existing example fixtures as an accepted release pack.
