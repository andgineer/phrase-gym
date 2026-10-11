# Phrase Gym — start here

Handoff updated **2026-10-11**. The complete project context is in `~/projects/phrase-gym/specs/`. Read this file before using any earlier plan or archive.

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
2. [Recording and comparison decision](decision-recording-and-comparison.md): the latest endorsed interaction flow, optional session recording, automated self/model playback and the unresolved stopping choice.
3. [Interaction design](design/interaction-design.md): concrete screens, controls and step-by-step examples, with agreed behaviour separated from visual proposals.
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
- Introduce each expression and its use before recall. Broad situations cannot demand an arbitrary exact sentence.
- Use enough meaningful, phrase-specific cue material from any part of the expression. Show necessary disambiguation initially. Deliberately vary hidden chunks when useful; no random masks or letter puzzles.
- The response is the whole phrase. Fixed targets have explicit equivalents; word/frame tasks define what may vary.
- Ambiguous cues and valid alternatives are not memory failures. Self-comparison remains available; AI/ASR grading is not required.
- Preserve support/exposure history. Reading a model, immediate echo, partial-cue retrieval, shown-word usage and silent practice are different evidence.
- One learning target has one schedule across examples, cue variants and dialogue appearances.
- Provide a session-level recording switch, self-recording/replay controls and automated comparison when recording is on. The endorsed working order is learner recording → complete phrase/model → self-check.
- Recording off permits practice when the learner cannot speak. It must not require microphone permission or imply demonstrated spoken performance; audio is manually requested in this mode.
- No deployment, Git initialization, course enrollment, echo-words output switch or Anki migration is authorized by this handoff.

## Still open

Manual recording stop was recommended; silence-based stop was offered as an alternative, but the user did not choose between them. Final visual styling, navigation, recording retention/defaults, platform/device feasibility, review policy numbers, first-source rollout and hosting also remain open. The user endorsed the recording proposal with “great”; this does not turn every previous mockup or technical assumption into a decision.

Keep the context focused: consult active documents first. The archives preserve historical material, including rejected ideas and misleading claims from the aborted implementation attempt, so they must never override this handoff.
