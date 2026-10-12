# P3 session planning and lifecycle

2026-10-12. The user authorized beginning P3 and committing directly to main. When asked to choose lesson eligibility, the user instructed the implementer to use the existing research to decide. Routine learning-policy defaults should be selected from that evidence without making the learner choose engineering parameters.

## Scope policy

All lessons and a selected lesson both select due/new targets within the requested learning language. An explicit lesson narrows eligibility; it does not reset due dates or create another schedule. The complete lesson list still includes non-due phrases. No Practice all control is added.

This adopts the existing recommendation in the [answer-card/review policy](design/answer-card-and-review-policy.md): preserve spaced retrieval rather than make early repetitions of every phrase the default. It is an engineering choice under the user's delegation, not a research claim that one lesson-selection interface has been experimentally proven best. Session budgets, new-item throttling and the FSRS adapter remain subsequent P3 work.

## Implemented foundation

- `sessions/planning.py` reads content and schedule versions inside the creation transaction. Due targets precede new targets; due targets are ordered by due time, ties and new targets by stable target ID. One target appears once even with multiple lesson/dialogue memberships.
- The initial cue selector uses the first reviewed cue in the immutable content revision. Persistent presentation, hidden-chunk coverage and difficult-chunk selection remain to implement with event processing. This batch does not claim adaptive cue selection.
- Required reference and cue media must be available. The manifest stores complete content/lesson snapshots, selected cues, hidden required chunks, review-group IDs, schedule versions and required media IDs. This payload is not an exposure receipt: the future client must record what it actually presents.
- One SQLite schedule row per target stores due time and version. An absent row denotes new/unassessed content at version zero. Session creation never inserts or advances a schedule; review policy state and atomic review receipts remain future work.
- `sessions/service.py` owns each serialized write. A stable session ID deduplicates starts, conflicting parameters fail, and retries return the current state with the original manifest. Pause/resume uses a version check with retry recognition. A paused session retains the single open slot.
- Explicit takeover names the currently open session. Creation and superseding are atomic; an empty/unready replacement leaves the old session intact. Superseded sessions remain readable and cannot resume. SQLite enforces the single open slot independently of application checks. This is the current single-user runtime; multi-user identity is not implemented.
- Lifecycle timestamps come from the injected server UTC clock. Client occurrence-time adjustment belongs to the future event pipeline.

The API exposes `POST /api/v1/sessions`, `GET /api/v1/sessions/current`, `GET /api/v1/sessions/{id}` and versioned `PATCH /api/v1/sessions/{id}`. Empty eligible queues return a descriptive conflict without creating a session. Generated OpenAPI/client contracts describe the exact shapes.

## Remaining P3 work

Start session/lesson drill controls stay disabled until the unified card and ordered event pipeline exist. Next implement original-response/support evidence, atomic idempotent review receipts and the FSRS adapter, then integrate the card, durable SQLite takes and IndexedDB pending journal. Completion and cursor checkpoints must follow acknowledged events; this foundation does not accept an arbitrary client completion/cursor. Repair queues, adaptive cue selection, lookup recordings and prepared dialogue rehearsal remain open.

P3 is in progress, not complete. AUDIO-01, actual Safari/headset and iPhone/PWA acceptance, real actor/Serbian content and reference listening quality remain open. No provider call, deployment, push or P2 review loop is part of this batch.
