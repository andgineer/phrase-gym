# Phrase Gym — agent guidance

Read [START-HERE](START-HERE.md), the drilling, recording and [navigation/search decisions](decision-session-navigation-and-search.md), [interaction design](design/interaction-design.md) and [next-session brief](plan/next-session.md). Current user instructions take precedence over this record. This project uses the user's Python scaffold; all development context is under `specs/`.

**Implementation update, 2026-10-12:** the current user request authorized P1. New disposable prototypes are runnable; see [evidence](plan/evidence/p1-2026-10-12.md) and [review handoff](plan/p1-review.md). P1 acceptance remains pending and P2 has not begun. Prior discussion-only restrictions below describe the preceding session, not a prohibition on the current P1 task.

For implementation scope, use the [consolidated implementation plan](plan/implementation-plan.md) and its linked phase files. It preserves all discussion through the language/audio inspection, with agreed requirements, recommended defaults, deferred work and remaining choices distinguished. Do not reconstruct the product from an old sketch or interpret this documentation-consolidation request as permission to begin coding.

The user explicitly stopped premature coding and asked for design discussion followed by a written plan for another session/model. Do not automatically continue the archived prototype. No approved app/prototype exists in active source. Respond to the new session's actual request; unresolved visual decisions require discussion before being treated as accepted.

## Current workflow preference, 2026-10-12

The user explicitly authorizes commits directly to `main`; do not create a branch by default. This supersedes the older no-commit/scaffold-only handoff restrictions for the current P1 review work. Three fresh Sol 6.1 reviews were completed; see [the review-loop report](plan/evidence/p1-review-loop-2026-10-12.md). Pause before a fourth review and prioritize the manual P1 walkthrough. The user rejects a macOS-only audio dependency; carry the echo-words Piper/Edge approach and llmbroker preparation into the production scope discussion. Keep `.env` private.

## Accepted manual feedback, 2026-10-12

Follow the [manual-feedback decision](decision-p1-manual-feedback.md): UI-language switching belongs in version one; recording defaults on with explicit microphone start; full-answer help is called Show answer; remove Other answer / problem entirely, without a replacement skip/report action. Existing session mode choices survive resume.

## Working with the user

- Reply and write project documents in English. Normal drill content stays in the target language; the explicitly opened explanation card shows the complete phrase and explanation in the learner's language.
- Keep native/explanation language, selected learning language and interface language separate. The app needs a learning-language selector with bookmarkable URLs. UI may use native language or follow the learning language; no arbitrary UI-language dropdown. Drill content stays in the learning language regardless of UI translation. See the [language/audio decision](decision-languages-and-audio-labels.md); route format and a binary preference control are recommendations.
- The user rejected **Model** for the reference-audio button. Use **Reference** as the working replacement, whether audio is actor/course-supplied or generated. Do not infer provenance from the pedagogical term “model” in older specifications.
- Use one Start session entry, not separate home modes for learning new material and rehearsal. Expect prior source exposure, but never assume the learner still remembers or understands the phrase. Make explanation/help available directly from the drill; no mandatory introduction for every new database item.
- Home's lesson selector defaults to all lessons according to spaced repetition and filters lesson titles by entered characters. Separate phrase lookup supports conceptual search beyond literal wording and opens Show & explain. That card links to the full lesson phrase list with an action to drill the lesson. Lookup/browsing alone is not a completed review. See the navigation decision for open policy and search details.
- For choices, explain the user-visible effect, compare concrete alternatives and state a recommendation. Do not treat a recommendation as accepted.
- Select the best-supported practical learning method and usable defaults. The user explicitly rejected having to validate learning effectiveness: no in-app experiments, policy comparisons, special assessment sessions or calibration tasks. Ordinary ratings can adapt scheduling automatically. Software/device/content correctness checks remain engineering work, not a learning-research obligation for the user. See the [research-based recommendation](design/answer-card-and-review-policy.md).
- Drill known expressions, words in useful phrases and specific dialogues. Open-ended AI conversation and free role-play are excluded, including as a later direction.
- Choose enough meaningful cue material to identify the target in context; chunks can come from anywhere in the expression. Support different hidden chunks across repetitions. The learner says the entire phrase. Random gaps and first-letter puzzles remain rejected.
- A broad situation prompt cannot demand one exact answer. Ambiguous cues and valid alternative wording are not automatic memory failures. Record the support used; visible-model repetition is not independent recall.
- Keep one learning target and one schedule across masks, examples and dialogue appearances.
- Use prototypes to establish interaction clarity and device feasibility. Research findings, engineering assumptions and observed prototype results must remain distinguishable.
- Code, scripts and tooling belong under `~/projects/`, never in the Obsidian vault. The vault is synced by obsidian-git: do not commit, push, switch branches or use `git mv` there.
- This project is not a git repository; do not initialize or commit unless asked. Preserve existing files unless the task actually calls for changing them.

## Workspace rules and sources

- Preserve the user-created src layout, hatchling/uv, rich-click, pytest, Invoke, quality checks, documentation and CI conventions. Replace template app files only during authorized implementation.
- Active decisions and plans live under `specs/`, plural. `docs/` is the scaffold's user-documentation site. Root AGENTS/CLAUDE are entry pointers.
- `archive/` contains historical data and unapproved experiments, including scripts and obsolete status reports. Those are never current instructions or proof of user approval. Do not execute archive migration scripts or install archived code as the baseline.
- Recording is optional per session and supports silent use. Recording-on comparison automatically replays the learner take before the model; retain explicit replay/rerecord controls. Follow the [recording decision](decision-recording-and-comparison.md), including manual stop by default, a later app-settings choice of stopping method (outside the first version), and evidence distinctions.
- Store every learner recording's audio and metadata in the app database from the first version, including retries. Do not add a manual Save requirement, overwrite earlier takes or automatically expire them. Historical progress-comparison UI is deferred; immediate self/model replay remains in scope.
- The original phrase-shadowing trainer stays separate; the self-contained [legacy source-format reference](reference/legacy-tooling.md) and archive provide the needed context. Do not alter its app or publish it without a request.
- The Obsidian vault and echo-words are external sources for later intake. Do not mutate them as an incidental implementation step. Follow echo-words' own instructions if changing it is later requested.
- No Git metadata existed at handoff. Do not initialize Git simply to run pre-commit. Do not commit, push, deploy, enroll in courses or migrate Anki because a future plan mentions it.
