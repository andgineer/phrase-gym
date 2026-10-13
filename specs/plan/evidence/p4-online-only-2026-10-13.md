# P4 online-only correction — 2026-10-13

Starting HEAD: `565ea45` (round-4 evidence; fixes `182b6df`). The latest explicit human instruction removes **all offline work** and authorizes one genuinely new Sol 6.1 round-5 review after correction, full gate, packaging and local commit. See the [authoritative scope](../../decision-online-only.md), [connected ownership model](../../p4-connected-state-model.md) and [runbook](../p4-runbook.md). Earlier offline decisions/model/review reports remain clearly superseded history.

## Result

Removed session downloads/packs/readiness, selected-media and shell caching, routine worker registration, install/update UI/assets, cached session/preferences/library restoration, offline reducers/projection/attempt IDs/support/grades, local cursor/completion/pause/resume and client repair deferral. Views map actual server attempts/receipts. New planner `due-new-v3-online` and generated contracts exclude retired repair options; narrow historical decoding preserves old payloads/receipts/queues without enabling a new offline trainer. Ordinary server repair policy and occurrence-time/original support semantics remain.

A small connection-error/retry composable hides normal learning/settings during known server failure/retry. New learning commands wait; active capture, permission requests and playback remain stoppable/cancellable. Retry resends original journaled writes, reloads real session/settings/library state and recovers pending transfers. Unrelated successful requests cannot alone dismiss the error. No health probes or general availability/reconciliation platform was added.

Every initiated event/start/transition and every take retains original identity/context/payload or bytes for acknowledgement; lookup exposures retain their separate localStorage records. Capture leases, application-lifetime volatile audio, old-card affinity/cleanup, exact-byte conditional deletion and SQLite receipts/scheduling transactions survive the removal. Owned old shell/media caches, matching registrations and obsolete pack/display fallback keys retire without wiping pending evidence/history or reloading active media. No operator data or external source was read or changed.

## Verification

The final ordered correction gate used Python 3.13.13 and bundled Node 24.19.0 LTS:

1. `uv run inv pre`: every hook passed; pyrefly zero errors; architecture and client formatting passed.
2. `uv run inv test`: **130 Python tests, 5 client tests and 206 browser checks** passed; client typecheck/build and 103 scenarios in each of Chromium/WebKit actually ran, without skips.

New behavioral coverage includes lost committed rating response without speculative progress; actual origin loss with capture/permission/playback stop or cancel; original-ID event/take reconciliation; real old-worker/populated-cache retirement while preserving disposable journals; online response/cache delivery and bounded historical payload compatibility. Existing connected recording/quota/ownership/conditional-ACK/navigation/conflict/original-support/dialogue tests remain. Solely withdrawn offline-success/projection/download tests were removed, explaining the lower counts than round 4.

Iteration failures were corrected before this gate: callback types, terminal permission-cancel observation, overlapping start retry, draft preservation/exposure retry and test locators synchronized to actual media/permission states. Earlier test assumptions about continuing normal UI after 503 were updated to the accepted connection screen while preserving journal/receipt assertions. The retirement fixture moved failures to the real server after WebKit proved page interception unreliable under a worker. Browser fixture startup uses module execution so both pytest collection and Playwright resolve helper imports. No failed iteration is passing evidence.

Both documentation languages built successfully. `uv run inv app-build` passed, including frozen dependency sync, clean npm install, client typecheck/build and wheel/source distribution. All five current client files match both distributions byte-for-byte; deleted production worker/manifest/projection/pack paths are absent. The archived old worker exists only as a browser test fixture. Fresh review has not yet occurred.

## Remaining boundary

This is supported connected engineering behavior, not an offline/eviction-survival or arbitrary simultaneous-device promise. Actual iPhone/trusted HTTPS, broader headset/longer real speech/interruptions and VoiceOver acceptance remain open. Safari/MacBook Pro M1 recording without clipping was positively reported by the human on 2026-10-13 for that setup only. P5 content/model/source work and P6 private hosting/backup/restore/rollback remain separate. No P5 implementation, provider call, push, publication or deployment was performed.
