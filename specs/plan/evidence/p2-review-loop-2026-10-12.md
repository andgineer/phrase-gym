# P2 fresh-session review loop — 2026-10-12

The user requested independent Sol 6.1 reviews of P2, analysis and fixes until no actionable findings remain, with a convergence stop after three iterations requiring an explanation and confirmation before further work. Each reviewer starts without conversation history, reads the current repository decisions and reviews the implemented P2 foundation. P1, future P3/P5 features and the acknowledged device/content acceptance gaps are outside this loop.

## Iteration 1

Reviewed baseline `126a1f0` using a fresh `gpt-6.1-sol` agent with high reasoning. The reviewer inspected backend and Vue responsibilities and reproduced defects using temporary databases and a disposable real-server Chromium session. Three findings were accepted:

1. Search route refreshes unmounted the inputs, losing focus after the first typed character. Keep the home inputs mounted while requests are pending, clear stale results and show loading rather than a provisional no-match claim. Browser coverage types through both queries and checks focus, including a held response.
2. Concurrent pagination appended duplicate pages and could hide final items. A focused pagination composable serializes each paginator and rejects responses from previous route versions. Pending buttons are disabled; deterministic browser coverage holds page requests and sends repeated clicks across lookup, complete lesson lists and lesson selection.
3. Dictionary key order changed content fingerprints and media IDs. Shared canonical JSON recursively sorts object keys while preserving array order. Compare stored content canonically, including legacy fingerprints. Media retries match existing canonical metadata before allocating an ID, preserving previously published asset references and repairing missing bytes. Integration cases cover unchanged reordered dictionaries, significant array order, genuinely changed media settings and legacy media IDs.

The lesson selector's accessible label also follows the existing UI locale rather than hard-coded English.

Verification after fixes: `uv run inv pre` converged with every hook passing and pyrefly reporting zero errors, then `uv run inv test` passed **37 Python, 4 client and 22 real-browser tests**. The original baseline also passed its full gate; these were behavior gaps in passing tests. One new array-order test initially assumed two existing cues; its fixture was corrected to supply two reviewed variants before checking that their order remains significant. No assertion was weakened.

## Remaining review work

Run iteration 2 against the fixed tree in another fresh Sol 6.1 session. Stop on a clean actionable review; otherwise assess and fix justified findings and continue within the user's iteration limit. This record is not real-device, linguistic/content or deployment acceptance.
