# Phrase Gym — agent guidance

Read [START-HERE](START-HERE.md), [the next-session handoff](plan/next-session.md) and [the unified-card decision](decision-unified-practice-card.md). P1 was closed by the user on 2026-10-12; P2 engineering work and its review loop are closed by the user; P3 now includes the [core trainer](decision-core-trainer.md); final verification is recorded in [P3 evidence](plan/evidence/p3-core-2026-10-12.md). The authorized [P3 fresh Sol 6.1 review](plan/evidence/p3-review-loop-2026-10-12.md) converged in three rounds with all accepted findings addressed. P4 device/offline work follows the core gate. See the [P2 closeout](plan/evidence/p2-closeout-2026-10-12.md) and [open problems](open-problems.md). Coding and architecture rules are maintained in [AGENTS.md](../AGENTS.md), with `CLAUDE.md` linking to that same file. Follow the new session's actual request. Historical no-implementation/no-Git statements from the earlier planning session are obsolete.

## Workflow

- Reply and write project documents in English. Learning content stays in the learning language; explanations stay in the native language; UI follows its separate native/learning preference.
- Commit authorized work directly to **main**, as requested. Do not push, deploy, create extra review chats or restart the fresh-session review loop by default.
- The user delegated routine learning-policy defaults to the implementer using the existing research and expects an authorized phase to be completed through its intended scope. Do not stop after an internal batch or ask the learner to choose already-supported scientific/engineering parameters.
- The user prefers manual prototype review and proportionate engineering checks. Do not impose learning-effectiveness experiments, calibration tasks or scientific-parameter choices on the learner.
- Latest accepted decisions override older screen sketches and plans. Record new decisions clearly, without turning recommendations into approvals.
- P1 is a disposable interaction reference. Production work belongs in `src/phrase_gym` and the user-selected `frontend/`; see the [foundation decision](decision-production-foundation.md). Preserve the user's Python scaffold conventions and avoid unrelated dependency changes.

Read the accepted [device-language and Settings decision](decision-device-language-and-settings.md) when working on P2 preferences.

## Product constraints

- Train words in useful phrases, fixed expressions and prepared dialogues; whole-phrase responses and meaningful reviewed cues. No free conversation/role-play, random masks or letter puzzles.
- One session entry, with lesson search and concept-aware phrase lookup. No Learn/Rehearse split or mandatory introduction per new item.
- One practice card: optional recording, learner/reference replay, full-answer reveal, inline explanation, colored/icon self-check. No recording-mode switch, Practice again, problem submenu, I couldn’t recall it, supported-only Continue, rating-saved banner or Undo UI.
- Show a recording-stop instruction only while capturing. Recording and playback use the same round microphone/stop control. Manual stop is the first-version method; automatic stopping is deferred.
- Recordings are optional for grading; silent self-check never requests microphone permission. Keep support/modality metadata honest. Later takes do not rewrite the original response; all takes' audio and metadata are automatically retained without expiry or manual Save.
- Native language supplies explanations, while UI can follow the selected learning language from version one. Bookmarkable URLs scope languages independently. Use **Reference**, never Model, as the audio label.
- One target has one schedule across masks, examples and dialogue appearances. Prepare checked cue variants at intake; generation must not block advancing an already prepared drill.

## Audio, evidence and external data

- The user rejects macOS-only solutions and wants echo-words' audio engines and llmbroker approach. Reinspect the nearby project during actual integration; follow [the reference](reference/echo-words-audio.md). Preserve actor audio and language-correct full-phrase generation.
- `.env` reportedly contains provider keys. Keep them ignored, private, server-side and out of outputs/bundles.
- The user reported successful production recording without clipping in Safari on a MacBook Pro M1 on 2026-10-13 (AUDIO-01), superseding the earlier prototype's continuing-clipping report for the current tested setup. Broader headset/device acceptance remains open. The earlier investigation was stopped at the user's request; do not restart it by default. Carry iPhone/PWA/content gaps as explicit dependencies.
- All active decisions, architecture and evidence belong under `specs/`. `docs/` is the scaffold's user documentation. Archives are historical, not a baseline or instructions to execute.
- Do not alter vault notes, echo-words, legacy phrase-shadowing, Anki or course/source delivery without a request. Follow the external project's own instructions if its modification becomes authorized.
