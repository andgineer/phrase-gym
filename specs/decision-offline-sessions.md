# Downloaded prepared sessions

2026-10-13. P4 extends the immutable P3 session/event protocol. This is prepared-session completion, including prepared dialogues, rather than offline library browsing, generation, imports or session planning.

## Download and storage

Download session selects the current manifest, its learning language, content/dialogue revisions and every required reference/partner asset. The production build installs a versioned app shell and manifest with PNG installation icons. Each selected asset must match the server's byte length and SHA-256 before the pack is marked verified. An incomplete download has visible progress/errors and cannot establish offline readiness. Browser storage persistence is requested best effort, never promised. Only explicitly selected media are cached; private learner audio remains in the IndexedDB transfer journal until acknowledged in SQLite.

A pack uses schema version 1 and preserves the server checkpoint independently from queued events. Preferences are cached for reopening the practice route. Events must be durably journaled before local application. A pure local projection supplies deterministic server-compatible attempt IDs, the first-response/support state, cursor and provisional results. It does not invent schedule receipts or due dates. Pending progress and locally finished results explicitly await server confirmation. A connected session without a downloaded pack retains P3's checkpoint/outbox behavior; only verified packs promise completion without the origin.

Downloaded queues remain stable: ratings in a downloaded session carry `defer_repairs`. Extra repair cards are deferred to the next scheduled practice rather than appended to this queue. Existing pinned repair steps still run. Ordinary connected, undownloaded sessions retain the five-minute/two-other-target repair policy. This routine reliability choice avoids predicting authoritative eligibility/schedules or introducing timed offline waiting; every original outcome still receives the normal server validation and scheduling policy.

Offline pause/resume is device-local and journaled independently of the server's active status. It prevents capture/practice on this device and survives reopening. Online transitions retain version checks. Switching language or opening a pack does not take over the server's open session or change any pinned binding.

## Reconciliation and updates

Foreground, an online signal, reopening and explicit Retry reconcile the original event IDs in sequence. Receipts are authoritative; a checkpoint is journaled before removing its acknowledged event. Checkpoint reads/writes compare sequence and session version in one IndexedDB transaction, and displayed state uses the retained checkpoint; delayed reads cannot replace newer acknowledged evidence. A lost receipt replays the same event. Conflicting events are copied into retained conflict records before leaving the active outbox, and authoritative state reloads. Historical stale-content/policy/schedule/session outcomes retain their server eligibility reason without advancing a schedule improperly. Audio transfer retries after receipts and independently of other rejected takes; ID/hash/size/completeness acknowledgements still gate deletion.

The service worker serves its complete shell, selected media and range responses. An updated worker waits for explicit activation at a paused/end checkpoint; pending events/takes disable the update action. App activation never automatically reloads a spoken attempt. Old shell assets remain cached for existing tabs. No schema migration deletes unacknowledged evidence; incompatible pack versions produce recovery guidance.

Quota, unavailable IndexedDB and evicted media produce visible failures. A journal write failure prevents a provisional rating/advancement; audio whose journal write fails remains volatile in the open tab, with an explicit keep-open/free-space/retry message. Missing cached audio blocks the offline promise until a new verified download. If browser storage itself is lost, the server can restore acknowledged history and recordings, but cannot reconstruct unacknowledged device bytes. No automatic expiration of pending evidence or browser-origin backup claim is introduced.

## Evidence boundaries

Real-server browser tests use disposable data and a severable loopback TCP proxy in both Chromium and WebKit. Playwright's WebKit `setOffline` rejects even service-worker-only responses ([upstream issue](https://github.com/microsoft/playwright/issues/42775)); disconnecting the origin transport exercises the actual cached shell/media without that emulation defect. This workaround preserves all offline assertions and skips no engine.

Actual iPhone Safari/installed mode, trusted HTTPS, speaker/headset, VoiceOver and real-speech interruption checks remain device acceptance dependencies. P6 hosting/backups and P5 model/source/content integration are separate work.
