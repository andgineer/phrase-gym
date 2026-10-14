# Recording, comparison and retention

Recording is optional on every attempt, including lookup/native-help practice with the full phrase visible. Silent self-check never requests a microphone. Capture starts explicitly and stops manually; show the stop instruction only during capture. Reuse the microphone stream between takes to avoid repeated device initialization, flushing recorder data before release. Pause, navigation, Settings, backgrounding and leaving the page release media; cancelled permission requests release late streams.

After a take, play the learner recording followed by the complete phrase's reference. Use Reference regardless of actor/course or generated origin, preserving provenance separately. Replaying or rerecording cannot change the original response's support or grade. Only one playback runs at a time.

## Keep every take

Automatically retain every separately identified take in SQLite, including imperfect, repeated, supported and recoverable interrupted takes. New takes never overwrite earlier ones, even when bytes match. There is no Save button or automatic expiry. Keep original bytes, declared MIME, checksum, capture facts and immutable target/revision/attempt or exposure context. No-audio failures remain events.

Learner bytes are opaque to the server: no decoding, container inspection, transcoding or external provider processing. Only the frontend plays them. Server decoding can reject valid phone captures despite successful browser playback and serves no storage requirement. Keep request/schema/base64 checks, nonempty audio, the 12 MiB size limit, MIME allowlist, context/revision checks and immutable-ID conflicts. Browser completeness records normal capture completion; the server neither infers it from bytes nor certifies playback compatibility. Reference-media validation is separate.

Journal pending bytes locally until a database acknowledgement matches ID, hash, byte count and completeness. An event receipt does not acknowledge audio storage. Stable IDs make retries idempotent; per-take errors retain bytes without blocking other transfers. Conditional deletion must preserve a newer capture snapshot. Stop at 8 MiB with an interruption warning to leave final-encoder headroom below the server limit; this is not silence detection.

Abrupt termination can recover journaled snapshots, not the unflushed tail. Verify capture ownership across tabs before marking an abandoned take incomplete. On quota failure, keep available volatile audio and tell the learner to keep the tab open. Browser storage is not a backup; consistent database backups must include every recording BLOB.

## Browser constraints

Use the emitted MIME, including browsers that supply none before capture starts. Reuse one media element for learner/reference playback and retries; cancellation invalidates earlier asynchronous playback. Feature-detect the optional [Audio Session API](https://www.w3.org/TR/audio-session/): recording for capture, playback for comparison/replay, auto on release. Browsers without a writable API use native behavior.

Journal event/checkpoint reads use their key prefixes so advancing a card does not clone unrelated audio buffers. Local checkpoint failure retains pending evidence and reports a storage error. Rejected requests show their validation reason; pending evidence alone does not imply a lost connection.
