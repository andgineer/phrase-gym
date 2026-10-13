# Phrase Gym — start here

**Latest product decision, 2026-10-13: Phrase Gym is ONLINE-ONLY. Follow [the authoritative decision](decision-online-only.md). Offline requirements are superseded. P4 connected engineering is closed after the fresh round-5 finding, independent targeted closure and final ordered gate. Actual-device/content/release acceptance remains separate. No further review round, P5 implementation, push or deployment is authorized in this task.**
Updated **2026-10-13**. P1/P2 are closed and P3 core implementation is complete. **P4 engineering is complete within the accepted online-only scope.** Fresh Sol 6.1 round 5 reported one P2 body-read finding; the SAME reviewer independently closed its correction at `102154b`. The final ordered gate passed every hook/zero pyrefly errors, **130 Python tests, 5 client tests and 208 browser checks** with typecheck/build and both engines executed. App-build and byte-for-byte wheel/source verification passed. See [the closeout evidence](plan/evidence/p4-online-only-2026-10-13.md) and [the independent report/closure](plan/evidence/p4-review-5-2026-10-13.md). P5 is the next separate phase; it has not started. Actual-device acceptance remains open.

The [three-round fresh Sol 6.1 P3 review](plan/evidence/p3-review-loop-2026-10-12.md) converged with 5 → 1 → 0 findings. That review passed 104 Python tests, eight client tests and 138 browser checks. The subsequent [local P3 follow-ups](plan/evidence/p3-local-followups-2026-10-13.md) include phrase hints, one-click session starts and conditional native help; the P3 baseline gate passes **110 Python tests, eight client tests and 164 browser checks**. This is the P3 baseline for the P4 reliability implementation.

Read [the next-session handoff](plan/next-session.md), [agent guidance](agent-guide.md) and [P3 verification evidence](plan/evidence/p3-core-2026-10-12.md). They contain the final interaction, production foundation, workflow preferences and unresolved work. Read [the P2 runbook](plan/p2-runbook.md) for actual run/import/check commands and [the foundation decision](decision-production-foundation.md) for module boundaries. No chat-history hunt is necessary.

P2 was closed at the user's request with [closeout evidence](plan/evidence/p2-closeout-2026-10-12.md). On 2026-10-13 the user reported production recording works without clipping in Safari on a MacBook Pro M1. [Open problems](open-problems.md) retain broader headset/device acceptance, untested iPhone behavior and incomplete real reference/content acceptance. Carry these forward; do not restart the P2 review loop or audio investigation by default.

## Current authoritative decisions

The [device-language and Settings decision](decision-device-language-and-settings.md) simplifies P2 preferences: URL-owned language, best-effort browser restoration and one Settings save at a time. It overrides earlier server-language write ordering/cache requirements.

The [unified practice-card decision](decision-unified-practice-card.md) is the latest interaction specification. It supersedes older separate cue/check/practice screens and intermediate manual-feedback designs.

- One practice card with optional recording, reference/replay and colored/icon self-check buttons. Recording or Show answer fills the gaps; explanation expands inline.
- Show answer and the visually distinct book-icon Show & explain action stay together on the card when useful native translation/explanation exists. Missing help hides its action and panel; no not-prepared placeholder is shown. The preparation prompt must supply a translation, adding explanation only when useful.
- The round microphone becomes a stop icon while recording/playing. Show **Press again to stop recording** only during capture.
- Ratings save and advance quietly. No recording switch, Practice again, problem submenu, I couldn’t recall it, supported-only Continue, microphone-ready filler, saved-rating banner or Undo UI.
- Native language supplies explanations; UI can follow native or learning language from version one. Learning-language URLs preserve separate sessions and bookmarks.
- Retain every learner take and the original response's support/evidence. Help does not become independent recall merely because a self-rating was supplied.

The product trains selected words in phrases, fixed expressions and prepared dialogues toward ready use. It is not an open-ended speaking partner. Train whole phrases using meaningful identifying cues and one schedule per target. No random masks, role-play, compulsory item introductions or learner effectiveness experiments.

## Workflow and implementation

- Commit directly to **main**. Do not create branches by default, push or deploy without a request.
- P1 manual review is finished for this session. Three fresh Sol 6.1 reviews were completed earlier; do not restart the loop. Use proportionate checks and user feedback.
- Production implementation belongs in the project's source layout during P2. The disposable prototype and archived experiments are not production baselines to transplant unquestioningly.
- The user rejects macOS-only audio dependencies and requests echo-words' Piper/Edge and llmbroker approach. See [the integration reference](reference/echo-words-audio.md). Existing Samantha clips are test assets only. Keep `.env` keys private and server-side.
- Earlier prototype testing reported Safari/M1/external-headset clipping; the 2026-10-13 production Safari/M1 test now reports recording without clipping. Headset details and broader acceptance remain unspecified; see AUDIO-01 in the open-problems record. iPhone checks and real actor/Serbian content also remain pending.

## Supporting documents

1. [Next-session handoff](plan/next-session.md) — concrete restart instructions and open dependencies.
2. [P1 closeout](plan/evidence/p1-closeout-2026-10-12.md) and [prototype run/walkthrough instructions](../prototypes/README.md).
3. [P3 core policy](decision-core-trainer.md), [P3 runbook](plan/p3-runbook.md), [P3 session planning](decision-p3-session-planning.md), [consolidated implementation plan](plan/implementation-plan.md), [P2–P3 work](plan/02-core-implementation.md), [architecture](architecture.md), [learning engine](architecture/learning-engine.md) and [data contracts](architecture/data-contracts.md). Older interaction descriptions yield to the unified-card decision above.
4. [Targeted drilling](decision-targeted-drilling.md), [navigation/search](decision-session-navigation-and-search.md), [language roles](decision-languages-and-audio-labels.md), [recording history](decision-recording-and-comparison.md) and [manual-feedback history](decision-p1-manual-feedback.md).
5. [Learning research](research-answer-card-and-scheduling.md), [review-loop evidence](plan/evidence/p1-review-loop-2026-10-12.md) and [original P1 evidence](plan/evidence/p1-2026-10-12.md).

Archived experiments and older planning-session restrictions are historical. They do not override the current user request or the latest decisions. Keep active project context in `specs/`; leave external vault, echo-words, legacy phrase-shadowing and Anki sources unchanged unless requested.
