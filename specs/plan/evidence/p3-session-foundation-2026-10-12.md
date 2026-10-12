# P3 session foundation — 2026-10-12

The first P3 batch implements pinned due/new session planning and durable lifecycle APIs. P3 remains in progress. See the [decision and remaining scope](../../decision-p3-session-planning.md).

## Verification

The closure gate ran in order:

1. `uv run inv pre`: all hooks passed after formatting converged; pyrefly reported zero errors.
2. `uv run inv test`: **76 Python tests, six client unit tests and 100 browser checks** passed. Client typecheck and build ran successfully. Browser checks covered the existing 50 client scenarios in both Chromium and WebKit against the real server and built client.

Ten new session integration scenarios cover manifest content/cues/media/groups, shared target versions, due/new scope, future exclusion from a selected lesson, absent media/empty queues, invalid language/lesson scope, unchanged complete lesson lists, immutable content after publication, restart recovery, idempotency conflicts, pause/resume retries and stale versions, explicit takeover and concurrent starts. The migration comparison verifies the new schema against SQLAlchemy metadata; an additional architecture case checks the sessions domain's HTTP boundary.

Session creation leaves schedules unchanged. Tests use synthetic reference bytes, temporary databases and an injected UTC clock. No provider calls, private recordings or deployment are involved. Existing dependency deprecation warnings did not fail checks.

## Limits

No new browser practice-card behavior is implemented in this batch; Start session/lesson drill remains disabled pending the event pipeline and UI. The API persists session lifecycle state and manifests, not attempt/cursor checkpoints yet. FSRS updates, recording retention/journaling, adaptive cue history, repairs and dialogue rehearsal remain P3 work. Existing browser tests establish regression coverage of the library/settings/media client, not an end-to-end production drill.

AUDIO-01, actual Safari/headset/iPhone/PWA behavior and real actor/Serbian content acceptance remain open in the [open-problems record](../../open-problems.md). P1/P2 review loops remain closed.
