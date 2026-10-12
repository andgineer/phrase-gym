# Next-session handoff — P1 closed

Updated **2026-10-12**. The user asked to finish P1 and continue in a fresh session. The disposable prototype is version **0.1.6**. P1 design/prototype work is closed at the user's direction; P2 production implementation has not started. Remaining device/content checks below are carried forward, not claimed passed.

## Read first

1. [START-HERE](../START-HERE.md) and [agent guidance](../agent-guide.md).
2. [Unified practice-card decision](../decision-unified-practice-card.md): the latest accepted interaction, including the final removal of rating confirmation/Undo. This overrides older contradictory screen and recording-mode descriptions.
3. [Consolidated plan](implementation-plan.md), then [P2–P3 implementation](02-core-implementation.md), [architecture](../architecture.md) and the relevant data contracts.
4. [Echo-words integration reference](../reference/echo-words-audio.md) and [prototype walkthroughs](../../prototypes/README.md).

No chat-history search is needed. Do not restart the three-iteration fresh-session review loop or rebuild P1. The user prefers manual prototype feedback and proportionate checks. Follow the next session's actual instruction before beginning P2.

## Final interaction to implement

- One practice card throughout: meaningful masked phrase/situation, recording and replay, Reference audio, Show answer, inline Show & explain, and three colored/icon ratings. Recording is optional on every attempt; self-check works without microphone permission.
- Recording or Show answer reveals the full phrase in place. Keep the controls. Show & explain sits beside Show answer, visually differentiated by a blue book-icon link, and expands phrase-specific meaning/usage/chunk details inside the card.
- One round audio control: microphone when idle, stop icon during recording, stop icon during playback. Stop playback returns it to a microphone. **Press again to stop recording** appears only during capture and disappears on stop.
- Rating buttons remain available before/after recording and after help: red repeat icon / Not yet, amber clock / With effort, green check / Recalled. Disable them during active capture/permission. A rating saves and advances quietly.
- **Removed:** recording On/Off switch, Practice again, Other answer / problem, I couldn’t recall it, Continue without a recall grade, idle microphone-ready message, saved-rating banner and Undo UI. Accidental-rating correction is deferred. Do not restore these from older plans.
- Retain the original response's evidence when rerecording. Save every take's audio and metadata, including recoverable interrupted takes, without manual Save or expiry. Replay uses the latest take. Supported ratings retain support metadata and must not masquerade as independent recall for scheduling.
- Native language supplies explanations. From version one, UI may use native language or follow the selected learning language; first-launch and Settings controls exist. Learning-language URLs are bookmarkable and scope sessions/search/lessons independently.
- One Start session, searchable lesson selection and separate concept lookup. Lookup opens phrase materials without grading; materials link to the complete lesson list. No mandatory introduction per new database item.
- Train selected words in useful phrases, fixed expressions and prepared dialogues; whole-phrase responses, reviewed meaningful masks, one schedule per target. No free conversation/role-play, random masks or learner effectiveness experiments.

## Workspace and production direction

- Commit directly to **main**, as explicitly requested. Current work is committed locally; do not infer a request to push or deploy.
- `prototypes/` is a disposable browser implementation/fixture/probe, not the production code location. `src/phrase_gym` still contains the user's Python scaffold. Build the real application in the project source layout during P2; agree a clear frontend location if the recommended Vue/TypeScript client is retained. Do not create another hidden/temporary implementation.
- Preserve the scaffold's uv/hatchling, CLI/package conventions, tests and CI where applicable. Vue/TypeScript PWA + Python/FastAPI/SQLite remains the recommended production stack, not installed functionality or a fully accepted platform decision.
- The user rejects macOS-only audio solutions and wants the same engines and llmbroker approach as nearby **echo-words**. Reinspect its current implementation when integrating: Piper/Edge, full-phrase caching, Serbian speech-only Cyrillic conversion, language-correct fallback, actor/course audio priority. Existing Samantha WAVs are disposable test assets, not a production dependency.
- The user reports copying provider keys into `.env`. Keep it ignored and server-side; do not print, commit or bundle secrets. No provider calls or external audio uploads were made in P1.
- P2–P3 implement the trainer and durable recordings; P5 handles intake/LLM preparation and source integration. Prepare checked cue variants at intake rather than making generation a dependency of each drill step.
- FSRS parameters, selected-lesson due-only/all policy, session budgets, initial source rollout and deployment details remain recommendations/choices. Do not present them as approved merely because a prototype exists.

## Open evidence and risks to carry forward

1. **Safari/headset clipping:** user reports recurring mid-take clipping on Safari, MacBook Pro M1, external headset. The prototype now reuses a microphone stream between takes, requests voice processing off, uses optional play-and-record Audio Session support, and releases on pause/navigation/background/completion. Synthetic Chromium/WebKit checks passed; the user has not confirmed the actual-headset fix. Verify this early in production audio work, and do not claim it resolved.
2. Actual iPhone Safari/installed-PWA microphone, headset, interruptions and persistence have not been verified. A phone-reachable trusted HTTPS origin is still needed for those checks. Treat these as release dependencies, not reasons to restart general prototype review.
3. Real actor/course clips and prepared Serbian reference audio remain incomplete; Serbian text/translation and listening quality need appropriate content checks. Twelve manual fixtures/two dialogues are examples, not a production content pack.
4. P1 uses IndexedDB and fixture review receipts, not durable server-side recording storage or production FSRS. Browser-origin data is not a backup. Concept search uses a hand-authored index; real retrieval quality is not established.

## Verification and running

The last substantive change passed **12 core checks, 9 browser walkthroughs, 16 regressions and 2 audio-continuity runs**. The final banner/Undo removal reruns the affected browser/regression suites; see the [closeout record](evidence/p1-closeout-2026-10-12.md). These are software checks, not real-headset or iPhone acceptance.

Run instructions are in [prototypes/README.md](../../prototypes/README.md). The current session used `http://127.0.0.1:8873/learn/en`; do not assume its process survives a session/app restart. Browser data stays with its origin. If needed, restart the documented server on the same port. No private recordings or secrets belong in the handoff.

Suggested next request: **Read specs/START-HERE.md and specs/plan/next-session.md, then begin P2 from this handoff. Commit directly to main.**
