# Learner recording storage

2026-10-14. The user explicitly requires the server to store learner recordings as opaque bytes. Only the frontend plays them; no server workflow needs to decode them. Remove server decoding, container inspection and decoding-status machinery from learner recording storage.

The actual iPhone test reached the private HTTPS server and acknowledged practice events, but four complete recordings were repeatedly rejected with HTTP 422: "Complete recording audio could not be decoded." The user reports that the same recordings play back on the iPhone. That unnecessary storage gate caused the reported failure.

## Accepted behavior

- Retain each separately identified take with its original bytes, declared MIME, capture time/settings, context and checksum. No decoding, container sniffing, transcoding or provider call occurs on this path.
- Keep request/schema validation, valid base64, nonempty audio, the 12 MiB size limit, supported MIME allowlist, revision/attempt/exposure binding, immutable-ID conflict checks and exact acknowledgement deletion guards.
- `complete` records whether browser capture finished normally. The server preserves that fact; it does not infer completeness from audio contents or claim playback compatibility. Interrupted takes remain incomplete.
- Frontend playback consumes the stored original bytes. Saving a completed take does not rewrite an attempt or advance a review schedule. Incomplete capture evidence retains its existing technical-failure behavior.
- Reference media publication has a separate validation/provenance workflow. This decision concerns learner recordings.

The four pending iPhone takes can retry with their existing IDs and payloads after the corrected server restarts. They need no fresh recording, tab reload, data clearing or metadata relabeling. Request/context/identity failures still retain device bytes for retry.

## Evidence boundary

The user reports local iPhone playback and the latest iOS, without a numeric version. Regression tests verify opaque and encoded byte retention, exact receipts, idempotent retries, capture facts and remaining integrity guards. Existing tests cover retention across session completion and server restart. Actual iPhone upload/replay must be retested before closing DEVICE-01; headset, interruption and VoiceOver acceptance remain separate device checks.
