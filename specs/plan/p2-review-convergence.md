# P2 review convergence — acceptance checklist

2026-10-12. **Accepted by the user.** The user approved the transition-coverage audit and fourth/fifth fresh Sol 6.1 reviews after the three-iteration stop. The audit and all review results are recorded in [the review evidence](evidence/p2-review-loop-2026-10-12.md). This checklist is part of P2 acceptance; it does not expand the product scope.

The fixes are holding; the reviews keep finding different interactions that completed-request walkthroughs did not exercise. Strengthen the acceptance description for loading, visibility, cancellation and persistence before another pass. Preserve the accepted P2 scope and existing architecture; do not restart P1, introduce P3/P5 features or treat device/content gaps as review defects.

## Acceptance invariants

| Responsibility | Explicit invariant | Required coverage audit |
|---|---|---|
| Query/navigation | Inputs remain usable while search refreshes; stale results cannot replace the current route or language. | Type through individual responses; hold an old response while changing query/language; retain Back/Forward queries. |
| Pagination | Each paginator has one pending page request; every matching item remains reachable exactly once. | Repeated clicks during a held request; at least three phrase pages; navigation before a page resolves; retry a failed page. |
| Language/lesson selection | Successful home and deep routes update bare-route restoration; a filtered selection never leaves a hidden lesson active. | Phrase/lesson bookmarks, setup completion, two explicit-language tabs, filter changes and cleared selection. |
| Shared preferences | Initialization has one authoritative read; ordered per-tab writes prevent an old request from undoing newer saved choices. | Navigate while initial settings are held; retry initialization failure; hold old language success/failure while selecting another language; overlapping settings submissions and reload. |
| Overlapping preference workflows | A settings-save response acknowledges its own fields without replacing a newer remembered route language in the tab cache. | Hold a full-save response, close settings, complete another language route, hold the post-save catalog refresh, then use history and bare-route restoration; check both cache-driven routing and persisted settings. |
| Reference playback | Loading playback already belongs to the single cancellable player; Stop, navigation and backgrounding cannot leave audio behind. | Hold media delivery; repeat the action; cancel before start; fail delivery; inspect genuine audio elements. |
| Reference evidence | Setup/configuration-hidden materials create no exposure. Retention precedes visible materials; acknowledgement does not block display/navigation. | Deep first-launch bookmarks, hidden configuration/history navigation, held/failed writes, navigation/reload and the same event's acknowledgement. |
| Import/media identity | Object key order is irrelevant; ordered content and real provenance/objective changes remain significant. Existing published IDs survive retries. | Legacy fingerprints and media IDs, reordered dictionaries, ordered arrays, changed metadata and recovery of missing original bytes. |

## Transition behavior

- During query loading, retain the input and its focus, clear stale results and show loading. A successful response supplies results or a genuine empty/no-match state. A failed request shows its retryable error without claiming no matches. Responses from a previous query/language cannot replace results or introduce an error on the current route.
- During pagination, disable that paginator's next-page control and retain loaded items. Success appends one page; failure retains the list and permits retry at the same offset. Navigation cancels the old response's authority, so its success/failure cannot alter the new language's list or pending controls.
- Pending audio start already owns the single player and exposes Stop. Stop, navigation and backgrounding cancel that player without a false playback error. Ended playback returns to Reference; failed delivery exposes an error and explicit retry. Missing prepared media remains distinct from a failed request and never triggers a different clip/language fallback.
- First launch and configuration hide phrase materials and create no exposure. For an eligible display, retain the immutable event before revealing. The write runs independently: pending shows saving, failure retains the event and offers retry, acknowledgement removes only that event. Local retention failure prevents reveal and offers recovery. Navigation does not discard an event for materials already displayed; an old acknowledgement cannot clear a later visit.
- Successful home/phrase/lesson routes update bare-route restoration while each tab's explicit language remains authoritative. Changing the lesson filter clears the previously selected lesson and its link. Stable content/media identities disregard object key order while retaining significant ordered arrays and real objective/provenance changes.
- Navigation during initialization shares the pending settings read; setup becomes available only after it completes, and a failed read allows explicit retry. All preference writes use one per-tab queue. While an older language write is pending, retain the latest queued route language; after the old success/failure, persist that latest choice. Native/UI settings submissions preserve their submission order. After acknowledgement and reload, the current saved choice remains authoritative.
- Full-save responses may contain a language fallback captured before a later route succeeds. Applying that response preserves the newer remembered language while acknowledging the saved native/UI preferences. Ordering server writes and reconciling the tab cache are separate requirements; history and bare-route navigation retain the same latest language in both.
- Settings controls expose their accessible names in the effective UI language, matching their visible labels in all four initial locales.

Coverage uses controlled request gates rather than arbitrary delays. Every browser case starts from a freshly reset disposable library/settings state; it then configures its own language and content. The reset endpoint exists only in the test server, never the production application. Real-device and linguistic acceptance remain separately labelled.

After the ordered `uv run inv pre` / `uv run inv test` gate, run each expressly approved fresh Sol 6.1 review of the complete current P2 implementation. Apply the same concrete-reproduction standard; do not manufacture findings to fill a checklist. A clean actionable review closes this loop. If further justified findings remain, report them and obtain confirmation before extending it again.

## Fifth-review stop

The user approved a fifth fresh review after the fourth-pass fixes. It found one remaining cross-workflow preference issue: a delayed full-save response could replace the newer cached language while queued writes kept the server correct. That response now preserves newer remembered choices, with combined save/navigation/history regressions. The final corrected tree still needs an independent clean review. Stop before iteration 6 and wait for the user's confirmation; see the evidence record for verification results.
