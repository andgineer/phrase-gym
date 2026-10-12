# P2 review convergence — proposed continuation

2026-10-12. **Pending user confirmation.** Three fresh Sol 6.1 reviews found 3, 3 and 1 actionable defects. Those seven findings are addressed and the full gate passes, but the final tree has not received a fourth independent review. See [the review evidence](evidence/p2-review-loop-2026-10-12.md).

The fixes are holding; the reviews keep finding different interactions that completed-request walkthroughs did not exercise. Strengthen the acceptance description for loading, visibility, cancellation and persistence before another pass. Preserve the accepted P2 scope and existing architecture; do not restart P1, introduce P3/P5 features or treat device/content gaps as review defects.

## Proposed additions to the P2 acceptance plan

| Responsibility | Explicit invariant | Required coverage audit |
|---|---|---|
| Query/navigation | Inputs remain usable while search refreshes; stale results cannot replace the current route or language. | Type through individual responses; hold an old response while changing query/language; retain Back/Forward queries. |
| Pagination | Each paginator has one pending page request; every matching item remains reachable exactly once. | Repeated clicks during a held request; at least three phrase pages; navigation before a page resolves; retry a failed page. |
| Language/lesson selection | Successful home and deep routes update bare-route restoration; a filtered selection never leaves a hidden lesson active. | Phrase/lesson bookmarks, setup completion, two explicit-language tabs, filter changes and cleared selection. |
| Reference playback | Loading playback already belongs to the single cancellable player; Stop, navigation and backgrounding cannot leave audio behind. | Hold media delivery; repeat the action; cancel before start; fail delivery; inspect genuine audio elements. |
| Reference evidence | Setup/configuration-hidden materials create no exposure. Retention precedes visible materials; acknowledgement does not block display/navigation. | Deep first-launch bookmarks, hidden configuration/history navigation, held/failed writes, navigation/reload and the same event's acknowledgement. |
| Import/media identity | Object key order is irrelevant; ordered content and real provenance/objective changes remain significant. Existing published IDs survive retries. | Legacy fingerprints and media IDs, reordered dictionaries, ordered arrays, changed metadata and recovery of missing original bytes. |

For each row, inventory existing behavioral coverage, fill justified gaps and identify concrete uncertainty. State the pending, successful, failed and cancelled behavior in the P2 functional description; use controlled request gates instead of arbitrary delays. Each browser case must establish the fixture/settings it needs and its expected evidence, rather than assume an earlier case completed a particular action. Keep real-device and linguistic acceptance separately labelled.

After the audit and the ordered `uv run inv pre` / `uv run inv test` gate, run **one additional fresh Sol 6.1 review** of the complete current P2 implementation. Apply the same concrete-reproduction standard; do not manufacture findings to fill a checklist. A clean actionable review closes this loop. If further justified findings remain, report them and obtain confirmation before extending it again.
