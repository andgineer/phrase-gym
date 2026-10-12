# P2 fresh-session review loop — 2026-10-12

The user requested independent Sol 6.1 reviews of P2, analysis and fixes until no actionable findings remain, with a convergence stop after three iterations requiring an explanation and confirmation before further work. Each reviewer starts without conversation history, reads the current repository decisions and reviews the implemented P2 foundation. P1, future P3/P5 features and the acknowledged device/content acceptance gaps are outside this loop.

## Iteration 1

Reviewed baseline `126a1f0` using a fresh `gpt-6.1-sol` agent with high reasoning. The reviewer inspected backend and Vue responsibilities and reproduced defects using temporary databases and a disposable real-server Chromium session. Three findings were accepted:

1. Search route refreshes unmounted the inputs, losing focus after the first typed character. Keep the home inputs mounted while requests are pending, clear stale results and show loading rather than a provisional no-match claim. Browser coverage types through both queries and checks focus, including a held response.
2. Concurrent pagination appended duplicate pages and could hide final items. A focused pagination composable serializes each paginator and rejects responses from previous route versions. Pending buttons are disabled; deterministic browser coverage holds page requests and sends repeated clicks across lookup, complete lesson lists and lesson selection.
3. Dictionary key order changed content fingerprints and media IDs. Shared canonical JSON recursively sorts object keys while preserving array order. Compare stored content canonically, including legacy fingerprints. Media retries match existing canonical metadata before allocating an ID, preserving previously published asset references and repairing missing bytes. Integration cases cover unchanged reordered dictionaries, significant array order, genuinely changed media settings and legacy media IDs.

The lesson selector's accessible label also follows the existing UI locale rather than hard-coded English.

Verification after fixes: `uv run inv pre` converged with every hook passing and pyrefly reporting zero errors, then `uv run inv test` passed **37 Python, 4 client and 22 real-browser tests**. The original baseline also passed its full gate; these were behavior gaps in passing tests. One new array-order test initially assumed two existing cues; its fixture was corrected to supply two reviewed variants before checking that their order remains significant. No assertion was weakened.

## Iteration 2

Reviewed `d579829` using another fresh `gpt-6.1-sol` agent with high reasoning. Three findings were reproduced in real Chromium and accepted:

1. A pending reference start was not considered active; repeated clicks created an untracked player that kept playing after navigation. Mark playback active before awaiting its start, so Stop, navigation and background cancellation retain ownership of the single player. Browser tests hold the actual media response, inspect genuine audio elements and verify pending-start cancellation without orphan playback or a false error.
2. Lesson filtering hid the selected option while its navigation link retained the old lesson. Changing the lesson filter now clears the selection and link; a browser case selects the next matching lesson and verifies its destination.
3. Phrase/lesson bookmarks did not become the last-used learning language. Remember the language after every successful supported content route and update the tab's cached fallback as well. Browser cases enter direct Serbian phrase and lesson URLs, observe persisted settings and verify restoration from `/`.

Verification: `uv run inv pre` passed every hook with zero pyrefly errors, followed by `uv run inv test`: **37 Python, 4 client and 32 real-browser tests passed**. An initial full-gate attempt found a TypeScript constructor-cast error in the genuine-audio instrumentation; the type was corrected and the complete ordered gate rerun. The playback tests explicitly restore the synthetic fixture reference binding rather than depend on previous editorial tests retaining it.

## Iteration 3

Reviewed `c2650dd` using a third fresh `gpt-6.1-sol` agent with high reasoning. One finding was accepted: explanation exposures could be recorded while first-launch setup hid the materials, or while an exposure write kept the materials behind Loading. Navigating away before the acknowledgement could therefore save support evidence for content never displayed.

First-launch/configuration states now defer content display and exposure creation. For an eligible phrase route, retain the event before revealing the materials and send its acknowledgement independently of route loading. A focused reference-exposure composable owns retention, pending/sending state and explicit retry; pending writes display a saving status rather than a false failure. Closing Settings reloads the current route. Browser cases cover the first-launch deep bookmark, configuration/history navigation and held exposure writes with and without navigation; visible visits retain their event until the real server acknowledges it.

Verification: `uv run inv pre` passed all hooks with zero pyrefly errors, then `uv run inv test` passed **37 Python, 4 client and 38 real-browser tests**. The main route composable is 220 lines; exposure and pagination responsibilities remain separate. No additional backend or architecture defect was established by this reviewer.

## Convergence stop

The three reviews produced **3 → 3 → 1 accepted findings**, all addressed. None of the later reviews reported an earlier fix recurring. The loop is narrowing, but it has not reached a clean independent review of the final tree; establishing that would require iteration 4. Stop here under the user's iteration limit and wait for confirmation.

The remaining convergence problem is coverage of interactions between loading, visibility, cancellation and durable evidence. Original tests largely followed completed requests and single actions; they missed ordinary typing, repeated actions, pending starts and setup-hidden content. The P2 plan describes the main flows but does not organize these transitions into a complete acceptance checklist. This calls for a focused state/failure matrix and coverage audit, not a new product scope or wholesale architecture change.

The user subsequently approved the [P2 convergence checklist](../p2-review-convergence.md), coverage audit and one additional fresh Sol 6.1 review. The three-iteration stop above is historical; the approved continuation is below. Actual-device, linguistic/content and deployment acceptance remain outside these checks.

## Approved transition-coverage audit

Existing coverage established normal navigation/history/tabs, per-keystroke search, three-page pagination, cancelled pending audio, setup-hidden materials, held exposure writes, reload recovery and canonical/legacy identities. Missing deterministic cases were added:

| Checklist area | Coverage after the audit |
|---|---|
| Query/navigation | `search.spec.ts` and `search-lifecycle.spec.ts`: typing/focus while pending, held old success/error responses followed by a new query or language, failed current query and explicit retry. Existing library walkthroughs retain Back/Forward search/navigation coverage. |
| Pagination | `pagination.spec.ts` and `pagination-lifecycle.spec.ts`: repeated clicks, all 120 distinct phrases/100 lessons, failure with same-offset retry and navigation before a held page response, across lookup, full lessons and lesson selection. |
| Language/lesson selection | `library.spec.ts` and `navigation.spec.ts`: independent tabs/bookmarks/history, first-launch deep bookmark, native/UI preferences, language restoration from phrase/lesson bookmarks and filter/selection coherence. |
| Playback | `reference-audio.spec.ts`: genuine audio players with held delivery, Stop/navigation/background cancellation, delivery failure and a successful explicit retry. Background coverage dispatches a controlled visibility event; it is not actual-device acceptance. |
| Exposure evidence | `library.spec.ts`, `reference-exposures.spec.ts`, `exposure-storage.spec.ts` and `exposure-write-lifecycle.spec.ts`: hidden setup/settings, held/failed writes, navigation/reload, local storage failure/recovery, exact-payload retry and an old visit's acknowledgement while a later visit remains failed. |
| Content/media identity | `test_content_identity.py`, `test_media_identity.py` and existing import/media integration cases: reordered keys, significant array order, legacy IDs/fingerprints, changed objectives/provenance and original-byte recovery. |

The audit reproduced two additional failure-state defects before fixing them: failed search claimed No matches, and a failed full-lesson page request hid the already loaded list and retry control. Empty/no-match text is now suppressed on failed queries, and lesson lists remain visible after page failures. The first targeted Chromium run passed 10 cases and failed these two assertions; after correction, the relevant 32 cases passed across Chromium/WebKit. Assertions were retained.

All browser cases now use an automatic fixture reset on the disposable server; settings, content revisions, exposures and media metadata do not carry between cases. Each test configures its own preferences; pagination packages use a shared responsibility-specific helper. Original synthetic reference bindings no longer depend on prior editorial test order. The test reset endpoint is only registered in `tests/browser_server.py`.

Audit verification: `uv run inv pre` converged with every hook passing and pyrefly reporting zero errors, followed by `uv run inv test`: **37 Python, 4 client and 70 browser tests passed**. Browser fixture isolation also removed the accumulated-content slowdown in per-keystroke search checks; every scenario passed in both Chromium and WebKit.

The approved fourth review follows. No push, deployment, P1 restart or real provider measurement is part of this continuation.

## Iteration 4

Reviewed audit commit `f1ac5ac` using a fourth fresh `gpt-6.1-sol` agent with high reasoning, after explicit user confirmation. The reviewer inspected the complete P2 foundation and ran all 37 Python tests. Two findings were reproduced against the real disposable server in Chromium and accepted:

1. Concurrent fire-and-forget language preference PATCHes could save Serbian, then let a held older English request overwrite it. The current tab remained Serbian, but a reload at `/` restored English. All preference mutations now use one per-tab ordered write queue; pending route-language changes coalesce to the latest choice. A failed older write does not strand the latest queued choice. Native/UI settings submissions share the queue and retain their submission order.
2. Concurrent initial settings GETs could return an old null-native response after setup had already saved a native language through another read. That late response reverted the UI to first launch while the database remained configured. Initial loads now share one pending read across route changes, exposing setup only after it completes. Initialization failure clears the pending read and allows explicit retry.

Seven new scenarios passed in Chromium/WebKit (**14 targeted browser checks**): held initial success/error with navigation and setup, held language success/error followed by Serbian or German (including coalesced intermediate choice), and overlapping native-language submissions with reload. The preferences adapter remains one focused module under 100 lines.

The fourth review did not report previous fixes recurring. The remaining gap was shared settings effects: route-response guards covered screen data, but initialization reads and background mutation ordering were outside the original transition audit. The checklist now states their ownership, ordering, failure and reload behavior explicitly. This continues to narrow the lifecycle coverage rather than change the learning product or its architecture.

The first full gate after these fixes passed Python/client checks and 83 browser cases, but an exposure-retry test read local storage before acknowledgement. Its error alert had disappeared because retry entered saving; that did not mean the save had completed. The trace showed the retry request still pending. The retry tests now await the matching server receipt, finished saving state and removal of the retained event, preserving the original assertion. The complete ordered gate was rerun after that correction.

Final verification: `uv run inv pre` passed every hook with zero pyrefly errors, followed by `uv run inv test`: **37 Python, 4 client and 84 real-browser checks passed**. The fourth-pass fixes are committed locally to main with their regression coverage and clarified acceptance rules.

The final tree has no clean independent review yet. Stop before iteration 5 and ask for confirmation of one further fresh Sol 6.1 review of the corrected P2 implementation, including shared settings lifecycle. No fifth review has been started.
