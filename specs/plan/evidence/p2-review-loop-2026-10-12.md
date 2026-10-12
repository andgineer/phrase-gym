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

The user subsequently approved one further fresh Sol 6.1 review. The fifth-pass results follow; the fourth-review stop is historical.

## Iteration 5

Reviewed `f2cfb6d` using a fifth fresh `gpt-6.1-sol` agent with high reasoning, after explicit user confirmation. The reviewer covered the complete P2 foundation, passed all 37 Python and four client tests and reproduced one finding in both Chromium and WebKit:

A held full Settings PUT response could replace the tab's cached last language after another successful route had remembered Serbian. Ordered language PATCHes saved Serbian correctly on the server, but the cache retained English. Holding the automatic post-save catalog refresh, returning through history to an invalid-language selector and following the brand to `/` restored English and then wrote it back to the server.

The preference adapter now records language-intention changes. A full save acknowledges its native/UI preferences while preserving a newer remembered language established after submission. The two-language browser regression holds both save response and subsequent catalog refresh, checks the actual server preference, then exercises history/bare-route restoration. Both Serbian and German cases failed on the prior build and pass after the change.

During integration, the parent also confirmed that SetupForm's hard-coded English accessible name overrode its localized visible native-language label. The control now uses the existing UI translation; all affected browser selectors were updated and four native-UI locale cases added. Those four cases failed before the fix. These are accessibility/localization checks, not learner effectiveness or real-device audio acceptance.

The new scenarios and existing preference initialization/write cases passed **26 targeted browser checks** across Chromium/WebKit. The preferences adapter remains focused at 97 lines. No provider, private-data, external delivery, push or deployment calls occurred.

The remaining convergence gap was an interaction between completed workflows: individual route guards and ordered server writes did not define how an old full-response snapshot should merge with newer cached state. The acceptance checklist now makes field authority/reconciliation explicit and requires the combined save/navigation/history sequence, checking client and server state together. This extends the existing P2 preference behavior without changing the learning product or introducing another service architecture.

Final verification: `uv run inv pre` passed every hook with zero pyrefly errors, followed by `uv run inv test`: **37 Python, 4 client and 96 real-browser checks passed**. Fifth-pass corrections are committed locally to main with their regression coverage and functional clarification.

The final tree has not had a clean independent review. Stop before iteration 6 and wait for confirmation of one further fresh review, including cross-workflow state reconciliation; no sixth review is authorized yet.

## Accepted simplification after iteration 5

The user questioned both the unchanged 37-test Python count and the growing preference coordination rules. A coverage audit showed that 37 was the complete backend foundation suite, while the client also had four unit cases and 48 unique browser scenarios run in both Chromium/WebKit. P3 sessions, scheduling and durable learner takes remain unimplemented; these counts never established their coverage. The backend measurement was 91.9% statements and 79.9% branches, with specific untested rejection/container/streaming paths.

The user approved the [device-language and Settings decision](../../decision-device-language-and-settings.md). The prior stop above and its proposed cross-workflow reconciliation review are superseded. Current language belongs to the URL; bare-route restoration is a best-effort browser preference. Server Settings now owns only native language and UI mode. Navigation makes no preference writes. One pending Settings save disables its fields, Save and toggle, retains failed drafts for retry, and leaves navigation available. The server PATCH endpoint, last-language contract field, per-tab write queue, coalescing and language-version response merge are removed. Migration 0002 removes the legacy JSON field while preserving native/UI settings. The preferences composable shrank from 97 to 65 lines, with a separate 20-line device adapter.

The last three review iterations concentrated on state ownership for an optional convenience. Simplification removes that cross-workflow requirement instead of extending its reconciliation matrix. Atomic publication, stable identities, immutable content/history and strict exposure retention remain unchanged; future grading and recording durability are not relaxed.

Added 28 meaningful Python cases: 12 invalid-reference/language/media-binding cases preserving published content, 10 compressed-container/provenance/base64/media cases, five streaming request boundary cases and one legacy Settings migration case. The MP3/Vorbis Ogg/AAC MP4 fixtures are checked-in synthetic tones, with generation commands and no private audio or provider calls. Existing queue-browser tests were replaced to match the accepted behavior: disabled/repeated submissions, failed-save retry, delayed save plus navigation, device isolation, explicit independent tabs and invalid/unavailable storage. Two client unit cases check best-effort storage and direct save suppression/retry.

Measured backend coverage is now **92.7% statements and 90.4% branches** (92.4% combined). Content contracts, package validation, media provenance contracts and the request boundary each measure 100%. The aggregate includes Alembic files reported as uncovered because its dynamic module loading is outside the configured package measurement; migration behavior is verified separately. This is automated path coverage, not linguistic, speech-quality or actual-device acceptance.

Commit **0fe7ec4** passed the ordered gate: every hook in `uv run inv pre` passed, pyrefly reported zero errors, then `uv run inv test` passed **65 Python, 6 client unit and 96 browser checks** (**48 distinct scenarios × Chromium/WebKit**) with client typecheck/build executed. No checks were skipped and no assertions were weakened. The same overall browser count now represents the simpler accepted behavior rather than the obsolete server synchronization invariant.

One bounded fresh Sol 6.1 review of this commit was dispatched after that gate; its result follows. No further generic review loop is authorized.

## Bounded review of the simplified design

A fresh gpt-6.1-sol reviewer with high reasoning reviewed commit 0fe7ec4 without prior conversation history, against the latest accepted decisions. It found one concrete P2 defect and no other actionable correctness findings: opening Settings hid the Reference controls without cancelling playback. A held media request could finish and start hidden audio without a Stop control. The reviewer reproduced this in Chromium; the parent added pending and active playback cases, both of which failed on the reviewed build in Chromium and WebKit (four failures).

Opening Settings now invokes the existing player cancellation operation. Closing Settings retains the existing route reload behavior. This is a small lifecycle correction, without restoring preference synchronization or adding a new invariant. Both pending/active cases are included in the full suite; the accepted checklist/runbook now state that hiding playback controls cancels the player.

The reviewer confirmed that the simplified preferences and migration match the accepted decision. It relied on the parent's completed full gate, independently reproduced the playback defect, and performed no repository edits or real-device/content/provider/deployment checks. This was the single approved review; the corrected tree is validated by regression tests and the required full gate, not claimed to have a second independent clean review.

Final ordered verification: every hook in `uv run inv pre` passed with zero pyrefly errors, followed by `uv run inv test`: **65 Python, 6 client unit and 100 browser checks passed** (**50 distinct scenarios × Chromium/WebKit**). Client typecheck and build ran. The four new playback regressions passed in both engines, retaining their failing-before-fix assertions. The bounded review is finished; no next iteration is planned. Changes are committed locally to main; actual-device/audio-quality/content acceptance remains open.

## User-requested phase closure

After reporting continuing mid-recording clipping, the user explicitly closed P2 while retaining that problem. The [P2 closeout](p2-closeout-2026-10-12.md) and [open problems](../../open-problems.md) are the current status. No further P2 review or audio investigation is planned without a new request.
