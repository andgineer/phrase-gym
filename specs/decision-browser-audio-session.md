# Browser audio and journal responsiveness

2026-10-14. The user reports silent learner playback followed by audible reference audio on the iPhone's built-in speaker, several seconds of disabled controls on advancement/Show answer, and a connection warning after recording. MacBook Safari works correctly. After loading the corrected frontend on the same iPhone, the user reports: "now it works on on iphone". This confirms the basic reported flow on that device; it does not establish headset, interruption or VoiceOver acceptance.

## Accepted browser behavior

Use the emitted audio MIME for each take, including browsers that report no recorder MIME before capture starts. Preserve original bytes and the existing exact acknowledgement rules. Only the frontend plays learner audio; no server decoding or conversion is introduced.

Reuse one media element for learner/reference playback and retries. Cancellation must invalidate earlier asynchronous playback operations. Feature-detect the optional Audio Session API: request recording mode for capture, playback mode for replay/comparison, and automatic mode when releasing media or leaving/backgrounding the page. Browsers without a writable API retain their native behavior. The [Audio Session specification](https://www.w3.org/TR/audio-session/) defines these modes; [WebKit's microphone routing report](https://bugs.webkit.org/show_bug.cgi?id=218012) supports checking routing but does not establish the cause on this user's phone.

Read journal entries by their key prefix so event/checkpoint operations do not clone unrelated audio buffers. Retain writes and require server acknowledgement as before; this adds no offline practice. Local checkpoint failures preserve pending events and report a storage error. Network/server failures keep connection recovery. Rejected requests expose their validation reason without treating every pending event as proof of a lost connection.

## Verification boundary

Browser regression checks use native MediaRecorder/playback and synthetic audio with disposable servers. They cover deferred MIME, non-silent retained samples, learner/reference order through one player, cancellation/retry, scoped journal reads and checkpoint recovery. They do not independently certify iPhone output routing or timing; the successful user retest is separate actual-device evidence, without numeric timings.

The private local test server writes its current logs to the ignored `.data/vault-test/server-debug.log`. The two diagnostic POST probes deliberately produced 403/422; those statuses are not evidence of failed phone uploads. Subsequent live practice events and recording uploads returned HTTP 200. The local and private HTTPS settings checks both returned HTTP 200, followed by the successful user iPhone retest.
