# Phrase Gym — start here

Handoff updated **2026-10-11**. The complete project context is in `~/projects/phrase-gym/specs/`. Read this file before using any earlier plan or archive.

**Current session, 2026-10-11:** the user asked to continue discussing app prototypes and functionality and to preserve all decisions in `specs/`. Design discussion has resumed; application implementation has not been requested. See the [current discussion record](design/interaction-design.md#resumed-prototype-discussion-2026-10-11). Continue recording accepted choices in the relevant decision documents and distinguish proposals from acceptance.

**Latest research/design request:** the user asked for academic evidence on learning words/phrases, an answer card and an effective spaced-repetition design. See the [evidence review](research-answer-card-and-scheduling.md) and [recommended answer card/review policy](design/answer-card-and-review-policy.md). **Accepted correction:** choose scientifically grounded usable defaults; do not ask the learner to validate effectiveness or add in-app experiments, special assessment sessions or calibration tasks. Exact three-button labels and FSRS settings remain recommendations, not user-approved choices.

## What the user wants

**Phrase Gym** trains selected words in useful phrases, fixed expressions and specific prepared dialogues until they can be used readily in the intended situations. The purpose is automatic access to useful language. It is not a general AI speaking partner: open-ended conversation and free role-play were explicitly rejected, including as future directions.

The user wanted design discussion in the current conversation, followed by a written plan for implementation in **a new session with a different model**. They explicitly stopped an assistant that began coding before discussing the design. Do not infer permission to resume that work from its existence or from older messages asking to “start.” The present handoff is documentation and project organization only. Follow the new session's actual user request for the next action.

Key user statements:

> “the app is to drill words in phrases or specific dialogs with specific phrases I should learn to the automatic usage in specific situations.”

> “it should for each specific phrase find what part is enough to show so it could not be confused with any other variants”

> “need control to record myself and hear … optional for the session … if it is on I want some automation here (auto play my version)”

> “after that you will write all our decisions to the plan for new session”

## Read in this order

1. [Accepted targeted-drilling decision](decision-targeted-drilling.md): scope, introductions, phrase-specific cues, varying hidden chunks, whole-phrase responses, alternatives, support and one schedule per target.
2. [Recording and comparison decision](decision-recording-and-comparison.md): the latest endorsed interaction flow, optional session recording, automated self/model playback and manual stop by default; the stop-method setting is deferred beyond the first version.
3. [Session navigation and search](decision-session-navigation-and-search.md), then [interaction design](design/interaction-design.md): home lesson selection, smart phrase lookup, lesson links and concrete screens, with agreed behaviour separated from visual proposals.
4. [Implementation roadmap](plan/implementation-plan.md): current status, scaffold constraints and future phases. [Next-session brief](plan/next-session.md) is the implementation handoff.
5. [Architecture](architecture.md), [learning engine](architecture/learning-engine.md), [data contracts](architecture/data-contracts.md): technical recommendations, not automatically approved decisions.
6. [Functional description](functional-description.md), [open choices](plan/functionality-discussion.md), and [research](research-memorizing-phrases.md): supporting scope and evidence limits.

## Current project state

The user created a Python scaffold under `~/projects/phrase-gym`. It contains `src/phrase_gym`, a rich-click CLI entry point, hatchling/uv packaging, pytest, Invoke, pre-commit/type-check tooling, documentation and CI/release workflows. The `main.py` file-conversion command and other cookiecutter examples are **templates to replace later**, not app behaviour. The scaffold's source, tests, dependency files and workflows were left unchanged by this consolidation.

There is **no approved Phrase Gym prototype or implemented app in the active project**. All roadmap phase gates remain unpassed. The old `phrase-shadowing/index.html` and `phrases.py` are legacy source tooling, not the new app and not progress to migrate.

The stopped attempt produced temporary source, fixtures and a browser prototype outside both project folders. It was never installed into this scaffold and its server was stopped. A labelled archive preserves that work so the context does not depend on `/private/tmp`; it is not a starting baseline, approved design or completed P1 milestone. The earlier visual sketches have the same non-authoritative status. See [archive inventory](archive/README.md).

## Decisions that must survive the handoff

- Name: **Phrase Gym**. Repository description: “A language trainer for automatic use of words and phrases through guided recall, spaced repetition, and targeted dialogue drills.”
- Train both words in useful phrases and fixed phrases/constructions; rehearse specific dialogues with defined targets. Source rollout remains a separate choice.
- The main screen starts one session; no separate Learn new material or Rehearse entry modes. Phrases are expected to have been encountered in source material. Start from the drill, with a visible action to show the complete phrase and an explanation in the learner's language when forgotten or unclear. No compulsory app introduction for every newly added item.
- Home has a searchable lesson dropdown, defaulting to All lessons under spaced repetition; title substrings such as `rap` find Building rapport. A separate phrase lookup also matches concepts absent from the wording, such as `greetings`, and opens Show & explain without starting a review. That card links to its lesson's complete phrase list, which has a Start lesson drill action. Selected-lesson due-only versus all-phrase policy remains open; see the navigation decision.
- Normal drills use the target language; the learner's language belongs on the explicitly opened explanation card. Broad situations cannot demand an arbitrary exact sentence. Keep track of explanation/model exposure when returning to practice.
- First launch with an empty database asks for native language and stores it in app settings for explanations. Russian is selected for the current sketch; it is not a hard-coded app default.
- Use enough meaningful, phrase-specific cue material from any part of the expression. Show necessary disambiguation initially. Deliberately vary hidden chunks when useful; no random masks or letter puzzles.
- When adding a phrase to the app database, have the LLM prepare alternate hidden-word/chunk cues where useful variants are possible. Check and store them with the phrase so a later session can show a different version. Do not force multiple variants or generate random gaps during practice.
- The response is the whole phrase. Fixed targets have explicit equivalents; word/frame tasks define what may vary.
- Ambiguous cues and valid alternatives are not memory failures. Self-comparison remains available; AI/ASR grading is not required.
- Preserve support/exposure history. Reading a model, immediate echo, partial-cue retrieval, shown-word usage and silent practice are different evidence.
- One learning target has one schedule across examples, cue variants and dialogue appearances.
- Provide a session-level recording switch, self-recording/replay controls and automated comparison when recording is on. The endorsed working order is learner recording → complete phrase/model → self-check.
- Show & explain also supports recording with the complete phrase visible throughout capture and learner/model comparison. These are model-supported practice attempts, stored like all other takes and linked to the same target/history. The session recording switch applies on both cards.
- Manual stopping is the default and the only stopping method in the first version. The compact drill starts capture with a microphone-icon button; during capture it becomes a stop icon plus **Stop recording**. Omit “Say the whole phrase aloud.” and “Ready when you are.” The complete-phrase task and automatic comparison remain unchanged. Choosing manual versus automatic stop belongs in later app settings; the selector and automatic stopping are deferred beyond the first version.
- Automatically store all learner recordings, including retries, as audio and metadata in the app database from the first version. No manual Save action or automatic expiry. A UI for comparing progress across historical recordings is deferred; immediate self/model comparison remains core.
- Recording off permits practice when the learner cannot speak. It must not require microphone permission or imply demonstrated spoken performance; audio is manually requested in this mode.
- No deployment, Git initialization, course enrollment, echo-words output switch or Anki migration is authorized by this handoff.

## Still open

Final visual styling, remaining navigation details, selected-lesson review policy, search implementation, recording on/off defaults, platform/device feasibility, review policy numbers, first-source rollout and hosting remain open. Manual recording stop and automatic retention of all recordings in the database are accepted. The stop-method setting, automatic-stop detection details and historical recording-comparison UI are deferred. The earlier endorsement of the recording flow does not turn every previous mockup or technical assumption into a decision.

Keep the context focused: consult active documents first. The archives preserve historical material, including rejected ideas and misleading claims from the aborted implementation attempt, so they must never override this handoff.
