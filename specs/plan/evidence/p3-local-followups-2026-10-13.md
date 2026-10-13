# P3 local follow-ups — 2026-10-13

Production baseline: **`4db59bf`** on `main`. P3 core engineering is complete, including the manual-use corrections below. The original [core evidence](p3-core-2026-10-12.md) and [completed fresh review](p3-review-loop-2026-10-12.md) retain their historical counts and findings.

## Corrections and verification

- Practice shows its situation instruction once and meaningful prepared partial phrase cues. The 44 local vault phrases have one manual cue each. Imports reject cues without visible phrase text; legacy content stays readable but cannot enter new retrieval sessions without a valid cue.
- Start session replaces the known open session in one click and retains its history. Resume keeps the existing position. Replacement waits for outstanding evidence receipts; lost start acknowledgements retry the same ID. The current-session resume banner is hidden inside that session.
- Show answer disappears once the answer is visible. Show & explain appears only when useful content exists in the selected native language, including after reload or Settings changes. Missing help has no placeholder panel or dead action.
- Translation-only help is accepted without mandatory usage filler. Generated OpenAPI/client types match this contract. Lookup without native help records full-answer exposure rather than claiming an explanation was displayed.

The ordered gate passed on this production baseline:

1. `uv run inv pre`: all hooks passed; pyrefly reported zero errors; architecture and client formatting passed.
2. `uv run inv test`: **110 Python tests, eight client tests and 164 browser checks** passed. Client typecheck and build ran. All 82 browser scenarios ran in Chromium and WebKit; none were skipped.

The user separately reports production recording works without clipping in Safari on a MacBook Pro M1. See [AUDIO-01](../../open-problems.md#audio-01--mid-recording-clipping) for the scope and remaining device evidence. Software tests use synthetic audio and establish neither real speech quality nor iPhone acceptance.

## Next phases

P4 can start from this baseline: device capture/playback/interruption acceptance, verified downloaded-session offline completion, service-worker/cache work, quota/eviction and reconnection/transfer resilience. Preserve the accepted unified card; do not restore a recording switch or closed review loop.

P5 still owns LLM preparation of genuinely different useful cues, native translations/necessary explanations, real source adapters and speech preparation. The [prompt-preparation plan](../phrase-preparation.md) is required full-app work; the current one-cue local pack does not complete it. P6 retains private deployment, measured hosting and verified backup/restore/rollback. These remaining phases do not block beginning P4 and are not completed by P3's engineering gate.
