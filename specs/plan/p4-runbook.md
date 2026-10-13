# P4 connected reliability runbook

Phrase Gym requires a live server. `uv run inv app-build` packages the current Vue client, favicon and a bounded retirement asset for the previous `/sw.js` URL. No downloads, installed offline app, cached browsing or local practice progression remain. See the [online-only decision](../decision-online-only.md).

1. Start/resume a session while connected. Server receipts own its position, support and results.
2. Temporary transport/server failure hides normal learning/settings and offers Retry. Stop or cancel audio remains available for an already initiated capture, permission request or playback.
3. Keep tabs with volatile recording storage errors open until retry succeeds. Pending events and every take retain their original IDs, bytes and capture context. Retry/foreground recovery resends original writes and reloads actual server state before enabling learning. No health probes or general outage platform is needed.
4. A conflicting event is retained locally as rejected evidence; inspect the current server card before making a new choice. Pending audio is deleted only after matching ID/hash/size/completeness acknowledgement. Current capture recovery uses Web Locks for ownership; older unleased capture remains retained with guidance to finish in its original compatible tab.
5. Earlier owned worker caches and obsolete packs retire without clearing IndexedDB evidence or reloading live media. Do not clear origin storage to upgrade. Browser journals are not backups or an eviction-survival guarantee.

The ignored `.data/vault-test` manual instance is unchanged by automation. Do not read or clear its progress, recordings or journals in tests.

## Engineering checks

Use Node 22.12+ on LTS. This session uses bundled Node 24.19.0 by prepending `/Users/andrei_sorokin2/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin` to PATH; the ordinary shell defaults to Node 20. Run `uv run inv pre`, then `uv run inv test` in order. Both browser engines use the real server and built client, disposable databases/profiles, synthetic capture and silent native playback. A severable TCP proxy checks representative server loss; an actual earlier worker fixture checks bounded retirement. No provider, operator microphone, private data or external source mutation is involved.

## Actual-device acceptance still required

Record device/OS/browser, equipment and outcomes. Actual iPhone Safari over trusted HTTPS still needs permission/cancellation, successive and longer spoken takes, comparison, speaker/headphones, background/phone interruptions, pause/resume and temporary server failure/retry checks. Verify VoiceOver, focus, enlarged text, touch targets, long/non-Latin cues and color schemes. These are device acceptance gaps, not requirements to restore offline installation. Keep personal takes out of fixtures/logs.

The 2026-10-13 Safari/MacBook Pro M1 report supports recording without clipping on that reported setup. Headset, duration and interruption coverage remain unspecified. P6 separately owns private hosting/access and backup/restore/rollback; P5 owns real reference/content acceptance and source/model preparation.
