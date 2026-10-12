# Next-session handoff — P2 foundation

Updated **2026-10-12**. P1 is closed and the production foundation is implemented in `src/phrase_gym` and `frontend/`. P2 is underway; P3 sessions, learner takes and scheduling have not started. The disposable interaction prototype remains version **0.1.6**. Remaining device/content checks below are carried forward, not claimed passed.

## Read first

1. [START-HERE](../START-HERE.md) and [agent guidance](../agent-guide.md).
2. [Unified practice-card decision](../decision-unified-practice-card.md): the latest accepted interaction, including the final removal of rating confirmation/Undo. This overrides older contradictory screen and recording-mode descriptions.
3. [Consolidated plan](implementation-plan.md), then [P2–P3 implementation](02-core-implementation.md), [architecture](../architecture.md) and the relevant data contracts.
4. [Echo-words integration reference](../reference/echo-words-audio.md) and [prototype walkthroughs](../../prototypes/README.md).

No chat-history search is needed. Do not restart P1's closed three-iteration review loop or rebuild P1. The user requested a separate P2 fresh-session review loop; its findings and current status are in the [P2 review record](evidence/p2-review-loop-2026-10-12.md). The user prefers manual prototype feedback and proportionate checks. Continue from the production foundation and the next session's actual request. See the [P2 runbook](p2-runbook.md), [foundation decision](../decision-production-foundation.md) and [verification evidence](evidence/p2-foundation-2026-10-12.md).

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
- `prototypes/` is a disposable browser implementation/fixture/probe, not the production code location. The production Python application extends `src/phrase_gym`; the user confirmed `frontend/` for the Vue/TypeScript client. Do not create another hidden/temporary implementation.
- Preserve the scaffold's uv/hatchling, CLI/package conventions, tests and CI where applicable. Vue/TypeScript + Python/FastAPI/SQLAlchemy/Alembic/SQLite is the installed production foundation. The service worker/offline PWA, phone acceptance and private hosting remain future work.
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

## Production foundation and next work

The requested P2 fresh-session review loop has reached its three-iteration stop: 3, 3 and 1 findings were addressed and the ordered full gate passed (37 Python, 4 client, 38 browser). The final fixed tree has not received a clean independent review. Wait for the user's confirmation of the [convergence proposal](p2-review-convergence.md) before a fourth review; see the [review evidence](evidence/p2-review-loop-2026-10-12.md). This pause applies to that review loop, not to a separately authorized task.

- Persistent native language and native/learning UI preference; `/learn/{language}` bookmarks; four initial UI locales; per-tab explicit routes win.
- Stable source-to-target identities, immutable content revisions, pinned dialogue bindings, staged/manual publication with stale-content rejection and atomic rollback.
- Language-scoped lesson filtering, concept/text lookup, complete lesson phrase lists and materials/exposure logging without recall grades.
- Hash-addressed original reference audio, provenance, exact text/language/role checks, byte ranges and missing/orphan reconciliation. Fixture audio used by tests is explicitly synthetic.
- Shared contracts are below HTTP. Focused services own validation, publication, catalog reads, settings and exposure. API routers and Vue views/composables are split by responsibility. `AGENTS.md` contains the adapted echo-words coding rules; `CLAUDE.md` is a symlink. Hooks enforce architecture direction, 300-line source limits and client formatting.

Next: finish real reference-media/content acceptance where material is available, then P3. Before implementing the selected-lesson session branch, resolve due-only versus all-phrase policy. Implement one unified practice card, durable SQLite recording BLOBs with an IndexedDB pending-take journal, honest original-response evidence, pinned sessions and the review policy. The existing pure reducer is an evidence foundation, not an implemented session engine. Start session/drill buttons are disabled until these services exist.

Use `uv run inv pre` then `uv run inv test` as the final gate. Do not rerun P1 or claim actual Safari/headset/iPhone acceptance from synthetic WebKit checks.
