# P3 fresh-session review loop — 2026-10-12

The user requested fresh independent Sol 6.1 reviews, analysis and necessary fixes until no actionable findings remain. After three iterations, further review requires a convergence explanation and user confirmation. Each reviewer starts without chat history and reads the current repository decisions. Closed P1/P2 investigations, actual-device clipping, P4 offline packs, P5 providers/content and P6 deployment remain outside this loop.

## Iteration 1

A fresh `gpt-6.1-sol` agent reviewed baseline `0554343` read-only across the Python and Vue P3 implementation. The baseline passed the ordered gate: 95 Python tests, eight client tests and 126 browser checks. Five findings were accepted:

1. Recording exposed the answer during permission/capture while retaining unsupported original evidence. Capture snapshot and completion reveal are now separate protocol events; unfinished capture remains technically ineligible across reload. Browser coverage holds permission, checks gaps during capture and verifies reveal after stop.
2. Old-card help/recording actions could queue behind a pending rating and poison the next card's event journal. Controls now disable during advancement; queued work binds the initiating session/step. Conflicting pending events remain separately retained locally, authoritative state reloads, and subsequent choices can proceed. A focused events composable owns the journal/action responsibility. Held-rating and competing-review browser cases cover this boundary.
3. History/bookmarks could display and address another same-language session. The route selects its own session independently of the launcher's open session; stale loads are guarded and session identity resets panel effects. Superseded bookmarks render retained results. Browser coverage takes over, navigates Back and resumes the actual open session.
4. Recovering incomplete audio did not retain technical exclusion. Pending capture evidence prevents grading before successful completion; incomplete persisted takes retain failure on the attempt without rewriting its original response. Backend and real-browser cases grade after interruption and assert no schedule promotion.
5. A permanently rejected recording stopped all later transfers. Each take now retains its own error/bytes while independent transfers continue. An 8 MiB capture resource boundary reserves headroom below the server's 12 MiB limit. Browser coverage retains an oversized take, records a valid one and checks database acknowledgement plus retained rejected bytes across reload.

The user also supplied three failing CI runs. The static job lacked Node/npm installation before the client-format hook; it now installs locked frontend dependencies before `uv run inv pre`. Linux Playwright WebKit lacked `MediaRecorder`; the complete browser job now uses macOS with both native engines. The docs build reproduced the `locale` API mismatch and a subsequent highlighter API mismatch; compatible mkdocstrings/PyMdown constraints and the updated lock select a toolchain that builds both documentation languages successfully.

Synthetic tone replay caused audible beeps. Test fixtures now set media output volume to zero before calling native playback; capture, decoding and playback events still execute. A capture regression verifies actual playback calls use zero volume. Tests use synthetic sources only.

During integration the parent also corrected late Reference acknowledgements starting hidden playback after navigation, with deterministic browser coverage. An ordering-dependent cue-history test incorrectly tried to take over a completed session; its setup now supplies takeover only for an open session and retains the original cue/history assertions.

First-round ordered verification passed every hook with zero pyrefly errors, followed by **96 Python tests, eight client tests and 138 browser checks**, with typecheck and build executed. Both documentation languages built successfully. No push, deployment or remote CI rerun is authorized. Local workflow corrections do not claim a new GitHub Actions run passed.


## Iteration 2

A second fresh `gpt-6.1-sol` agent reviewed the complete first-round tree without conversation history. It established one further finding: recent help was dated from cue presentation/finalization rather than its actual exposure. After leaving a card open ten minutes, revealing the answer and immediately taking over, the new attempt could incorrectly receive independent FSRS credit.

Recent support now derives its time from the existing acknowledged reveal/help event receipts, preserving original responses and avoiding a duplicate timestamp authority. Eight integration cases cover Show answer, explanation, reference and recording completion, with immediate takeover and expiration after the actual exposure window. The four immediate-exposure cases failed before correction; the four expiration cases already passed.

Second-round ordered verification passed every hook with zero pyrefly errors, followed by **104 Python tests, eight client tests and 138 browser checks** (69 scenarios in Chromium and WebKit), with typecheck/build executed. No checks skipped.


## Iteration 3 and convergence

A third fresh `gpt-6.1-sol` agent audited the complete corrected P3 tree and its behavioral coverage, with no prior conversation history. It returned **no actionable findings** after reviewing transactions, support clocks, immutable evidence, recordings/context/acknowledgements, FSRS exclusions, repairs, dialogue queues, planning/readiness, route ownership and capture/playback/journal lifecycle. This reviewer ran no tests or browser servers and made no edits; the parent's full ordered gate above establishes engineering verification.

The independent findings narrowed **5 → 1 → 0**. All accepted findings are addressed, no earlier fix recurred, and the loop converged within three rounds. No fourth review or convergence approval is needed. Documentation-only handoff updates receive the same configured source-size, formatting, hygiene and type-check hooks before the local commit. Actual headset/iPhone, content/provider, offline and deployment evidence remains outside these results.
