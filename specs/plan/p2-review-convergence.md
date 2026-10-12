# P2 review convergence — acceptance checklist

2026-10-12. **Accepted by the user, then simplified after five reviews.** The [device-language and Settings decision](../decision-device-language-and-settings.md) replaces server fallback synchronization. Earlier findings and fixes remain historical evidence in [the review record](evidence/p2-review-loop-2026-10-12.md). This checklist covers P2, without P3/P5 or real-device/content acceptance.

The latest three reviews concentrated on coordinating an optional convenience across navigation and Settings. Remove that coordination requirement instead of adding more queue/cache rules. Retain essential publication, content identity, reference evidence, request cancellation and library behavior.

## Acceptance invariants

| Responsibility | Explicit invariant | Required coverage audit |
|---|---|---|
| Query/navigation | Inputs remain usable while search refreshes; stale results cannot replace the current route or language. | Type through individual responses; hold an old response while changing query/language; retain Back/Forward queries. |
| Pagination | Each paginator has one pending page request; every matching item remains reachable exactly once. | Repeated clicks during a held request; at least three phrase pages; navigation before a page resolves; retry a failed page. |
| Language/lesson selection | Supported language URLs remember a best-effort device fallback; a filtered selection never leaves a hidden lesson active. | Phrase/lesson bookmarks, setup completion, two explicit-language tabs, filter changes and cleared selection. |
| Settings | Initialization shares one read; only one save is pending per tab. Navigation never writes server preferences. | Held initial success/failure with navigation; disabled save/fields/toggle; duplicate submit suppression; failed-save retry and persisted native/UI preference. |
| Device language | Explicit URLs win; bare URLs use only a valid browser value. Storage failure never blocks language navigation. | Deep bookmarks, two tabs, separate device contexts, invalid/blocked storage, delayed Settings response plus language navigation. |
| Reference playback | Loading playback already belongs to the single cancellable player; Stop, navigation, opening Settings and backgrounding cannot leave audio behind. | Hold media delivery; repeat the action; cancel before start; open Settings during pending/active playback; fail delivery; inspect genuine audio elements. |
| Reference evidence | Setup/configuration-hidden materials create no exposure. Retention precedes visible materials; acknowledgement does not block display/navigation. | Deep first-launch bookmarks, hidden configuration/history navigation, held/failed writes, navigation/reload and the same event's acknowledgement. |
| Import/media identity | Object key order is irrelevant; ordered content and real provenance/objective changes remain significant. Existing published IDs survive retries. | Legacy fingerprints and media IDs, reordered dictionaries, ordered arrays, changed metadata and recovery of missing original bytes. |

## Transition behavior

- During query loading, retain the input and its focus, clear stale results and show loading. A successful response supplies results or a genuine empty/no-match state. A failed request shows its retryable error without claiming no matches. Responses from a previous query/language cannot replace results or introduce an error on the current route.
- During pagination, disable that paginator's next-page control and retain loaded items. Success appends one page; failure retains the list and permits retry at the same offset. Navigation cancels the old response's authority, so its success/failure cannot alter the new language's list or pending controls.
- Pending audio start already owns the single player and exposes Stop. Stop, navigation, opening Settings and backgrounding cancel that player without a false playback error. Ended playback returns to Reference; failed delivery exposes an error and explicit retry. Missing prepared media remains distinct from a failed request and never triggers a different clip/language fallback.
- First launch and configuration hide phrase materials and create no exposure. For an eligible display, retain the immutable event before revealing. The write runs independently: pending shows saving, failure retains the event and offers retry, acknowledgement removes only that event. Local retention failure prevents reveal and offers recovery. Navigation does not discard an event for materials already displayed; an old acknowledgement cannot clear a later visit.
- Supported language URLs remember a best-effort browser value while each tab's explicit language remains authoritative. Changing the lesson filter clears the previously selected lesson and its link. Stable content/media identities disregard object key order while retaining significant ordered arrays and real objective/provenance changes.
- Navigation during initialization shares the pending settings read; setup becomes available only after it completes, and a failed read allows explicit retry. Settings saves native language and UI mode only. Pending disables its editable fields, Save and toggle. Success applies those two preferences and closes configuration; failure retains the draft and releases controls. A duplicate submission sends no second request.
- Language restoration is browser-local and optional. Bare routes use a valid stored language or show choice; invalid or unavailable storage never prevents explicit language navigation. Settings responses contain no learning language, so there is no navigation-write ordering or fallback reconciliation.
- Settings controls expose their accessible names in the effective UI language, matching their visible labels in all four initial locales.

Coverage uses controlled request gates rather than arbitrary delays. Every browser case starts from a freshly reset disposable library/settings state; it then configures its own language and content. The reset endpoint exists only in the test server, never the production application. Real-device and linguistic acceptance remain separately labelled.

After the ordered `uv run inv pre` / `uv run inv test` gate, run one bounded fresh Sol 6.1 review of the simplified accepted P2 implementation. Fix concrete essential defects within this scope. Record the outcome and stop the generic loop; do not manufacture findings or repeatedly extend review.
