# P3 session planning and lifecycle

2026-10-12. The user authorized beginning P3 and committing directly to main. When asked to choose lesson eligibility, the user instructed the implementer to use the existing research to decide. Routine learning-policy defaults should be selected from that evidence without making the learner choose engineering parameters.

## Scope policy

All lessons and a selected lesson both select due/new targets within the requested learning language. An explicit lesson narrows eligibility; it does not reset due dates or create another schedule. The complete lesson list still includes non-due phrases. No Practice all control is added.

This adopts the existing recommendation in the [answer-card/review policy](design/answer-card-and-review-policy.md): preserve spaced retrieval rather than make early repetitions of every phrase the default. It is an engineering choice under the user's delegation, not a research claim that one lesson-selection interface has been experimentally proven best. Session budgets, new-item throttling and the FSRS adapter remain subsequent P3 work.

## Current implementation

The 2026-10-13 manual correction makes Start session and selected lesson/dialogue starts complete in one click. The start action itself authorizes replacement of the known open session, whose history remains retained; there is no second Start selected session instead step. Resume session retains the existing session and position. Viewing another language or returning to the library still makes no replacement request. The server keeps explicit current-session ID checks and idempotent creation; unknown concurrent changes remain conflicts.

The initial batch implemented pinned manifests, idempotent creation, versioned pause/resume, explicit takeover and one open SQLite session slot. The [core trainer decision](decision-core-trainer.md) now supersedes its provisional first-cue/unbounded-queue behavior: persistent cue history, a bounded due/new budget, FSRS, ordered original-response/support events, durable takes, the unified card and prepared dialogue rehearsal are implemented.

The [P3 runbook](plan/p3-runbook.md) and [core verification record](plan/evidence/p3-core-2026-10-12.md) own current use/evidence. Initial foundation verification remains preserved in its [evidence record](plan/evidence/p3-session-foundation-2026-10-12.md). P4 device/offline, P5 preparation/source and P6 deployment/backup work remain separate. AUDIO-01 and actual headset/iPhone/content acceptance remain open; no provider calls or deployment are part of this batch.
