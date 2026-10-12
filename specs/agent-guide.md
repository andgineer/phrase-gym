# Phrase Gym — agent guidance

Read [START-HERE](START-HERE.md), [the next-session handoff](plan/next-session.md) and [the unified-card decision](decision-unified-practice-card.md). P1 was closed by the user on 2026-10-12; P2 has not begun. Follow the new session's actual request. Historical no-implementation/no-Git statements from the earlier planning session are obsolete.

## Workflow

- Reply and write project documents in English. Learning content stays in the learning language; explanations stay in the native language; UI follows its separate native/learning preference.
- Commit authorized work directly to **main**, as requested. Do not push, deploy, create extra review chats or restart the fresh-session review loop by default.
- The user prefers manual prototype review and proportionate engineering checks. Do not impose learning-effectiveness experiments, calibration tasks or scientific-parameter choices on the learner.
- Latest accepted decisions override older screen sketches and plans. Record new decisions clearly, without turning recommendations into approvals.
- P1 is a disposable interaction reference. Build production work in the project's source layout when P2 is requested. Preserve the user's Python scaffold conventions and avoid unrelated dependency changes.

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
- Safari/M1/headset clipping remains unconfirmed on the actual device after a mitigation. Synthetic WebKit results are not Safari/headset acceptance. Carry iPhone/PWA/content gaps as explicit dependencies.
- All active decisions, architecture and evidence belong under `specs/`. `docs/` is the scaffold's user documentation. Archives are historical, not a baseline or instructions to execute.
- Do not alter vault notes, echo-words, legacy phrase-shadowing, Anki or course/source delivery without a request. Follow the external project's own instructions if its modification becomes authorized.
