# Phrase Gym — start here

Updated **2026-10-12**. **P1 is closed as a prototype/design phase at the user's request. P2 has not started.** The final prototype is version **0.1.6** under `prototypes/`; `src/phrase_gym` remains the user's Python scaffold. Device/content limitations remain open evidence, not passed checks.

Read [the next-session handoff](plan/next-session.md) and [agent guidance](agent-guide.md). They contain the final interaction, production direction, workflow preferences and unresolved work. No chat-history hunt is necessary.

## Current authoritative decisions

The [unified practice-card decision](decision-unified-practice-card.md) is the latest interaction specification. It supersedes older separate cue/check/practice screens and intermediate manual-feedback designs.

- One practice card with optional recording, reference/replay and colored/icon self-check buttons. Recording or Show answer fills the gaps; explanation expands inline.
- Show answer and the visually distinct book-icon Show & explain action stay together on the card.
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
- The user reported Safari/M1/external-headset clipping. A repeated-stream/audio-processing mitigation passed synthetic tests, but actual-device resolution is unconfirmed. iPhone/PWA checks and real actor/Serbian content also remain pending; carry them into later work explicitly.

## Supporting documents

1. [Next-session handoff](plan/next-session.md) — concrete restart instructions and open dependencies.
2. [P1 closeout](plan/evidence/p1-closeout-2026-10-12.md) and [prototype run/walkthrough instructions](../prototypes/README.md).
3. [Consolidated implementation plan](plan/implementation-plan.md), [P2–P3 work](plan/02-core-implementation.md), [architecture](architecture.md), [learning engine](architecture/learning-engine.md) and [data contracts](architecture/data-contracts.md). Older interaction descriptions yield to the unified-card decision above.
4. [Targeted drilling](decision-targeted-drilling.md), [navigation/search](decision-session-navigation-and-search.md), [language roles](decision-languages-and-audio-labels.md), [recording history](decision-recording-and-comparison.md) and [manual-feedback history](decision-p1-manual-feedback.md).
5. [Learning research](research-answer-card-and-scheduling.md), [review-loop evidence](plan/evidence/p1-review-loop-2026-10-12.md) and [original P1 evidence](plan/evidence/p1-2026-10-12.md).

Archived experiments and older planning-session restrictions are historical. They do not override the current user request or the latest decisions. Keep active project context in `specs/`; leave external vault, echo-words, legacy phrase-shadowing and Anki sources unchanged unless requested.
