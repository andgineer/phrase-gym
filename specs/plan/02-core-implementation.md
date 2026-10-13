# P2–P3 — production foundation and core trainer

Updated **2026-10-13**. P2 engineering is closed at the user's request. P3 core engineering implementation is complete and its final gate passed: [110 Python tests, eight client tests and 164 browser checks](evidence/p3-local-followups-2026-10-13.md), with all hooks and client typecheck/build executed. The production application extends `src/phrase_gym` and `frontend/`. Latest [unified-card decisions](../decision-unified-practice-card.md) override historical switches, split screens, problem submenus and Undo.

## P2.1 — Application skeleton and contracts

Implemented: source scaffold, same-origin client/API, settings/language routes, native/learning UI locales, shared contracts and generated client types. See [P2 evidence](evidence/p2-foundation-2026-10-12.md) and [production boundaries](../decision-production-foundation.md).

## P2.2 — Durable content and identities

Implemented: immutable revisions, stable source identities, atomic manual publication, scoped lesson/concept lookup, complete lesson lists, pinned dialogue bindings and idempotent reference exposures. P5 adds real intake/preparation sources.

## P2.3 — Reference media

Implemented: original hash-addressed bytes/provenance, exact text/language/role validation, delivery/ranges, missing/orphan reconciliation. Actual actor/Serbian speech and device acceptance remain open; see [open problems](../open-problems.md). Run/import commands remain in the [P2 runbook](p2-runbook.md).

## P3.1 — End-to-end session

Implemented: bounded due/new planning for All lessons or one lesson, immutable manifests, explicit resume/takeover, language pinning, authored chunk masks, one practice card, optional recording, latest-take/reference playback, inline native-language explanations and quiet ratings/results. Lookup recordings use exposure contexts rather than invented sessions. IndexedDB preserves pending events/takes and acknowledged checkpoints; SQLite retains every take, including recoverable incomplete captures. Device capture/replay effects are isolated from rendering and the evidence reducer.

## P3.2 — Review and transaction correctness

Implemented: immutable original-response/support snapshots; ordered, idempotent event receipts; payload conflicts; stale session/content/policy/schedule guards; server-bounded occurrence time; one target schedule/review-group update; versioned FSRS population parameters at 0.90 retention with no library short steps; next-day unassessed learning checks; persistent cue/chunk coverage and reported-cue exclusion. A repair queue requires five minutes plus two other finalized targets, caps extra retrievals at two, and defers short sessions without waiting. Supported practice, technical problems and corrections never overwrite the original difficulty.

## P3.3 — Specific dialogue rehearsal

Implemented: pinned complete exchanges, actual script exposure, prepared partner audio/supporting lines, target cues, isolated-turn detours and restored dialogue positions. Repeated targets share their group/schedule. The full exchange remains available in results; no open-ended chat is added.

## P3.4 — First useful local build

Library, ready/preparing due counts, scoped session entry, due times, optional capture, explanation, recordings and scheduling are integrated against reviewed manual packages without a model provider. Empty/preparing collections, unavailable backend, pending storage, lost acknowledgements, interruption and reload retain evidence. Self-check never requests microphone permission; a capture failure creates no automatic forgetting grade. The administrative cue-report event retains ambiguity without reintroducing a learner problem/skip submenu.

The [core trainer decision](../decision-core-trainer.md) owns lasting policy and evidence rules; the [P3 runbook](p3-runbook.md) owns actual run/use commands. Source, contract and real-browser checks establish software behavior, not language-content effectiveness, actual headset/iPhone acceptance or backups.

## Remaining work outside P3

- **P4:** actual Safari/headset/iPhone/PWA and interruption acceptance, downloaded-session offline completion, quota/eviction/transfer hardening and trusted phone access. AUDIO-01 remains open; its investigation is not restarted here.
- **P5:** reviewed real content, source adapters, add-time LLM cue preparation, explanations/search metadata and multilingual reference speech. Preserve source read-only boundaries and budget/semantic-review requirements.
- **P6:** private hosting/access, measured resources, consistent backups and restore/upgrade/rollback evidence.

No learner-effectiveness experiment or closed P1/P2 review loop is restarted. Commit directly to main; push/deploy still require an explicit request.
