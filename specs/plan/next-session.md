# Brief for the next session

Read [START-HERE](../START-HERE.md), the [drilling decision](../decision-targeted-drilling.md), the [recording decision](../decision-recording-and-comparison.md), and the [interaction design](../design/interaction-design.md) before doing anything from an older implementation plan.

The user is ending the design/planning conversation and intends to start a new session with a different model. **No application implementation is requested as part of this handoff.** In the new session, follow the user's actual request. Do not resume the archived implementation automatically.

## Product to build when implementation is requested

Phrase Gym drills specific words in phrases, fixed expressions and bounded prepared dialogues toward ready use. It introduces the model first, cues the intended expression with sufficient meaningful material, asks for the whole phrase, provides self-comparison and repair, varies useful hidden chunks and revisits targets over time. Alternatives/ambiguity are handled without punishing valid English. One target has one schedule.

The most recent design decision adds optional session recording. With recording enabled, automatically play the learner's take, then reveal/play the model, then offer self-check and rerecording. With recording off, allow silent practice using Reveal & compare and manually requested model audio. Stop-by-button versus stop-on-silence is still an open choice; manual stop was recommended.

## Preserve the user's scaffold

- Python package: `src/phrase_gym`; command entry point: `phrase_gym.main:phrase_gym`.
- Keep hatchling/uv packaging, the version file, pytest, Invoke, existing quality checks, documentation and CI/release workflows unless a specific implementation need justifies a change.
- `src/phrase_gym/main.py` is still a cookiecutter file-conversion template. Replace it with useful app behaviour at the appropriate implementation phase; it is not evidence of an existing app.
- All development context belongs under **`specs/`**, plural. `docs/` is the scaffold's user-documentation site. Do not create a competing `specs/` tree.
- Production architecture is proposed, not installed: Vue 3 + TypeScript/Vite, FastAPI/Python, SQLite, local media, one API process and a bounded preparation worker. Confirm platform suitability through the device prototype.
- No Git repository was present when this handoff was made. Do not initialize Git merely to make pre-commit run. Record direct-check limitations honestly.

## First useful steps

1. Review the concrete screen flow with the user wherever a substantive choice remains unresolved. Do not treat archived HTML or a widget selection as approval. The core drilling and recording requirements need not be reopened.
2. After the user requests implementation, start P1.1: a small reviewed cue/material pack covering fixed expressions, words, frames, competing answers and two prepared dialogues.
3. Create the P1 interaction prototype against the agreed flow, including the recording-on automatic comparison and recording-off silent path. A text-only sketch cannot complete that gate.
4. Build and run P1.3's real audio/storage probe with actual model audio and a private development HTTPS route. Record real iPhone results separately from desktop checks.
5. Record the user's walkthrough corrections and device evidence, settle the production starting point in P1.4, then follow P2–P6 from the roadmap.

## Do not inherit the stopped work as completion

The previous assistant prematurely wrote a disposable text-only prototype, fixture pack and CLI/tooling changes in a temporary directory. The user stopped it and required discussion first. No such code or dependency changes were applied to the active scaffold. Its archive is for provenance/recovery only, with no approval or phase credit.

Its useful diagnostic findings, if encountered again: the supplied lockfile may need reconciling with the manifest; a staged Zensical/mkdocstrings combination failed with a `locale` argument error and was fixed there by a newer mkdocstrings. These are historical observations, **not instructions to upgrade the current project**. Reproduce any issue in the real implementation before changing dependencies. The recorded test passes covered only the unapproved text prototype, not recording, the new design or the scaffold in its current state.

## What does not need another external context hunt

The research sources and their limitations, product decisions, architecture, detailed phase plans, source-tooling format, and inspected echo-words integration constraints are all under `specs/`. Raw discussion and earlier experiments are in [the archive](../archive/README.md). Actual vault lesson audio and the separate echo-words codebase remain external source systems for later integration; they are not copied into the application or authorized for mutation by this handoff.
