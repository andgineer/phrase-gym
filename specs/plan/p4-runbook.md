# P4 reliability runbook

Use the built client with the normal Python server; `uv run inv app-build` includes the worker, manifest and installation icons. Service workers require a secure context: loopback works for local automation; a phone requires a trusted HTTPS origin. The existing ignored `.data/vault-test` instance is unchanged by P4 automation. Do not clear its progress, recordings or journal to test failure cases.

1. Start/resume a prepared session while connected. Select Download session and wait for Verified for offline practice. Missing assets and storage failures prevent that confirmation.
2. Disconnect the origin, reopen the same bookmarked practice URL, use help/reference/optional recordings, pause/resume, and finish. Locally finished results await server confirmation. New session planning and arbitrary library access require connectivity.
3. Reconnect and foreground the app, or use Retry beside pending progress/transfer errors. Inspect the authoritative results after confirmation. Conflicting actions remain retained locally; revisiting another language never relabels them or performs takeover.
4. At a pause/end checkpoint, Update app appears when a newer worker is waiting. Resolve pending events/takes before activation. Keep tabs with volatile recording storage errors open until retry succeeds.
5. If downloaded media was evicted, reconnect and download again. A lost browser journal cannot be reconstructed beyond server-acknowledged history. Browser data are neither a database backup nor permanent storage. Current capture/recovery requires Web Locks for exclusive take ownership. An older unleased capture remains retained with guidance to finish in its original capture tab/compatible app; the new app does not claim an unresponsive legacy writer or discard its bytes.

Earlier version 1 packs require a fresh download while connected under the current version 2 protocol. Pending events and audio stay retained and can reconcile independently; do not clear storage to upgrade a pack. Script support expires for later distinct targets after five minutes, while support already observed on an attempt and its original response remain immutable. Test both acknowledged and queued dialogue introductions through offline reopening and reconnect; transient checkpoints must not create fresh script exposure.

## Engineering checks

Use Node 22.12+ on an LTS release. This session used the bundled Node 24.19.0 by prepending `/Users/andrei_sorokin2/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin` to PATH; the ordinary shell defaults to Node 20. Run `uv run inv pre`, then `uv run inv test` in order. Browser fixtures use temporary databases and persistent browser profiles with synthetic capture. Offline tests sever a disposable TCP proxy rather than Playwright WebKit's defective offline-emulation path. Both engines exercise real service-worker navigation, verified audio/ranges, IndexedDB, event receipts and SQLite. No provider, operator microphone, private data or external source mutation is involved.

## Actual-device acceptance still required

Record device/OS/browser versions, install mode, input/output equipment and outcomes. On the actual iPhone, test Safari and installed mode independently over trusted HTTPS: permission denial/ignored-request cancellation; at least three successive spoken takes; a longer continuous take; learner/reference comparison; speaker and headphones; background/phone interruption; pause/resume; offline close/reopen and reconnection with matching take counts. Confirm VoiceOver names/focus and support/pending states, enlarged text, touch targets, long/non-Latin cues and both color schemes. Do not share personal takes or put them in fixtures/logs.

The 2026-10-13 Safari/MacBook Pro M1 report supports recording without clipping only on that reported setup; unspecified headset, duration and interruption coverage remain open. No phone-reachable host is provisioned by this phase. P6 separately owns host/access selection and backup/restore/rollback evidence; P5 owns real reference/content acceptance and source/model preparation.
