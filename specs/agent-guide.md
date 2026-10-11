# Phrase Gym — agent guidance

Read [START-HERE](START-HERE.md), the two decision documents, [interaction design](design/interaction-design.md) and [next-session brief](plan/next-session.md). Current user instructions take precedence over this record. This project uses the user's Python scaffold; all development context is under `specs/`.

The user explicitly stopped premature coding and asked for design discussion followed by a written plan for another session/model. Do not automatically continue the archived prototype. No approved app/prototype exists in active source. Respond to the new session's actual request; unresolved visual decisions require discussion before being treated as accepted.

## Working with the user

- Reply and write project documents in English. Normal drill content stays in the target language; the explicitly opened explanation card shows the complete phrase and explanation in the learner's language.
- Use one Start session entry, not separate home modes for learning new material and rehearsal. Expect prior source exposure, but never assume the learner still remembers or understands the phrase. Make explanation/help available directly from the drill; no mandatory introduction for every new database item.
- For choices, explain the user-visible effect, compare concrete alternatives and state a recommendation. Do not treat a recommendation as accepted.
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
