# P4 review continuation — awaiting authorization

2026-10-13. Three fresh Sol 6.1 reviews found **3 → 2 → 1** accepted actionable findings. Each round exposed a boundary omitted by earlier coverage: application-lifetime volatile audio and lifecycle restoration, concurrent capture ownership and delayed support ingestion, then expired dialogue support in provisional results. The third-round correction also exposed an offline-introduction reconnect race while adding its regression. All accepted findings are implemented; the final gate and local commit are recorded in the [review-loop evidence](evidence/p4-review-loop-2026-10-13.md). This is not a zero-finding review or actual-device acceptance. No fourth review is authorized yet.

## Functional-description changes

- Recovery owns bytes at application scope and uses an exclusive per-take lease, conditional current-entry transactions and separate namespaces for incompatible writers. A timeout cannot prove a recorder dead; byte-exact acknowledgement cannot delete newer audio.
- Lifecycle restoration records explicit active/paused intent. First disconnected actions and empty-outbox reloads are acceptance cases, alongside completion and authoritative closure.
- Support is an occurrence-time fact. Checkpoints retain script-exposure chronology after acknowledgement; local projection applies the same closed five-minute window for later distinct targets. Original responses remain immutable. Pack version 2 requires a fresh download of earlier packs while preserving their pending evidence.
- Restoration applies queued actions before displaying a checkpoint. Transient raw introductions cannot create script exposure or run card actions during reconciliation.

These changes are in the accepted [offline decision](../decision-offline-sessions.md), not speculative reviewer recommendations. They tighten the functional description around facts the client must retain and display.

## Proposed next review, only after confirmation

If the user authorizes continuation, create one new local Sol 6.1 chat to review the full committed P4 result independently. Include a focused boundary matrix: first action after origin loss; zero-outbox restoration; acknowledged versus queued introduction; distinct dialogue targets on both sides of support expiry; reopen before/after completion; foreground and another tab; legacy/current capture and pack protocols; quota failure during finalization; delayed acknowledgement with newer same-size bytes; and waiting updates with retained evidence. Reproductions use controlled clocks and native ownership with real server/built-client Chromium and WebKit checks. Do not infer parity from mocks or fixtures with repeated target identities.

Evaluate each finding, implement accepted corrections, then run `uv run inv pre` followed by the full `uv run inv test` and commit locally. A zero-finding result establishes review convergence. If the additional review finds actionable issues, report them and seek a new scope decision rather than silently extending the loop. No learner-effectiveness experiment, P5 provider/source work, P6 deployment or external-data modification follows from this plan.

Actual iPhone Safari/installed-PWA, trusted HTTPS, headset/speaker, real speech/interruption and VoiceOver checks remain separate acceptance dependencies. A desktop WebKit gate does not close them.
