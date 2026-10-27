# Phrase Gym — agent instructions

Read [specs/START-HERE.md](specs/START-HERE.md) and [remaining work](specs/plan/implementation-plan.md). Specs retain current decisions and hard-to-infer design constraints; code and tests describe implemented behavior. The application is a small local Python service with the retained Vue client in `frontend/`. Earlier Python and Rust code in Git are prototypes, not parity requirements.

For a root-session request to implement the next stage, follow the [orchestration handout](specs/plan/orchestration-handout.md) before making changes or delegating. Delegated agents follow their assigned scope and the regular rules below.

`CLAUDE.md` links to this file; keep one instruction source.

## Architecture and file structure

- Keep architecture under active review in every batch. Each module and service has one coherent responsibility. Split unrelated workflows before adding more methods; do not grow a catch-all service or application component.
- Aim for no more than 250 lines of handwritten production code per file; review and split before 300. Generated contracts and migrations may be longer. A shorter file does not justify circular dependencies, meaningless wrappers or fragmented invariants.
- Dependencies point inward: transport and UI adapters use contracts/domain services; domain services never import HTTP routers, application startup or UI modules. Python owns the generated API contract. Construct services at the application boundary. Keep Phrase Gym lightweight and Coach separate under the [application boundary](specs/architecture.md#application-boundary); do not recreate the prototype's general distributed worker or native Coach GUI.
- Keep validation, publication transactions and library reads separate. One workflow owns each transaction; helpers receive that transaction instead of opening hidden writes. Keep all package publication atomic.
- Vue views render state. Route orchestration, preferences and audio effects belong in focused composables/adapters; the practice reducer stays pure. Generated API types are the client contract.
- Preserve the original Vue shell, practice card, microphone control and responsive styling as the Python backend grows. Adapt or hide controls for unavailable workflows; do not replace the interface with a new design.
- When moving a symbol, update all importers and tests. No re-export compatibility shims. Shared helpers are preferable to duplicated logic.

## Commands and environment

| Task | Command |
|---|---|
| Python lint, format, types, module size, client format and contract drift | `uv run inv pre` |
| Python tests, client typecheck/unit/build and both browser engines | `uv run inv test` |
| Python iteration | `uv run pytest` |
| Client iteration | `npm --prefix frontend test` |
| Browser iteration | `npm --prefix frontend run test:browser` |
| Refresh OpenAPI and generated client types | `uv run python scripts/generate_api.py && npm --prefix frontend run api-types` |
| Build the retained client | `npm --prefix frontend run build` |
| Run the local application on 127.0.0.1:8000 | `uv run phrase-gym` |
| Build the frontend and run the local application | `uv run inv run --build` |
| Install the pre-commit hooks (once per clone) | `uv run pre-commit install` |
| Scaffold tasks and release commands | `uv run inv --list` |
| Build documentation | `uv run inv build-docs` |
| Build the client and Python distributions | `uv run inv build` |

Use `uv sync --frozen` and `npm --prefix frontend ci`; install browser binaries from `frontend/` with `npx playwright install chromium webkit`. Python 3.13 and application dependencies are pinned in `uv.lock`. Use Node 22.12+ on an LTS release (`.nvmrc`). The pre-commit framework runs the pre gate and needs uv, Python, Node and the client dependencies; `inv pre` runs the same hooks manually. Dependency changes update their committed lockfiles. Piper uses its separately pinned private environment described in [the import guide](docs/importing-phrases.md#piper-voice). Missing dependencies or browsers are environment failures to fix, not reasons to skip suites.

Preserve the repository scaffold: pre-commit, Invoke tasks, version/tag helpers, package metadata, environment/build scripts, documentation publishing, test/coverage reports and PyPI release workflows. Extend these for the application instead of replacing or deleting them. Only application/documentation boilerplate should be replaced with real implementation and documentation. Follow [development and release operations](docs/development.md).

Pre-commit owns the static checks and all Ruff settings. Use Pyrefly for Python type checking; do not add a duplicate mypy gate or a wrapper that reruns the same tools. Invoke owns build, test and server commands; keep root-level executable scripts limited to the retained environment activation script.

Run the full browser/audio gate on macOS and retain the separate Linux Python server job in CI, following the [platform boundary](specs/architecture.md#application-boundary). Do not skip native recorder tests to accommodate Linux WebKit's missing API.

## Done gate

Choose the gate from the complete change, including staged and unstaged files.

**CI results are always part of verification:** inspect GitHub Actions for the current remote commit before making changes. After the local gate passes, commit and push under the standing authorization below, then wait for the workflows on the exact pushed commit to finish and inspect every result before delivery. Inspect failed jobs and their logs, fix failures, and repeat the applicable checks and push until CI passes. Local checks do not establish that CI passed. Report the commit, run link and outcome separately from local checks. Missing, pending, cancelled or inaccessible runs are unverified, never passing; report any actual block explicitly.

**Documentation-only changes:** when the change contains only prose in specs, plans, README, docs or agent instructions, do not run application builds or Python/client/browser suites; the installed pre-commit hooks still run. Run `git diff --check` and `git diff --cached --check`, inspect the full diff, verify changed local links/anchors and check consistency with related decisions and plans. Report these documentation checks without claiming application tests passed. Documentation describing future behavior does not require implementing or testing that behavior.

**Implementation or mixed changes:** changes to production code, tests, runtime prompts, generated contracts, migrations, dependencies, executable scripts, build configuration, hooks or CI require the full gate, even when accompanied by specs. Before describing such a milestone as done, ready, fixed, landed or green, run these in order and inspect their complete results:

1. `uv run inv pre`: pre-commit hygiene, Python formatting/lint/types, module-size, client formatting and generated-contract drift checks all pass.
2. `uv run inv test`: Python, client and both real-backend browser suites pass; frontend checks must run rather than skip.

Use focused checks during iteration. Run the full ordered gate once per coherent implementation deliverable before commit or delivery, rather than after every internal edit. Rerun relevant checks for review fixes and the full gate after substantive changes; a narrow subset cannot replace it. Use the documentation-only gate for prose-only changes. Never bypass commit hooks with `--no-verify`.

Fix findings even when they appear pre-existing. Deferral requires evidence that the same failure exists on the committed baseline and the user's agreement in the current turn. Never silently report a failing tree as passing. If execution or approval infrastructure blocks a check, describe that block explicitly and do not claim the gate passed.

## Tests

- New modules/functions ship meaningful behavioral coverage in the same commit. Keep tests with the responsibility they verify; do not grow one catch-all test file.
- Behavior that only exists in the browser needs a real-browser test against the real server and built client. Assert provisional/error states deterministically; do not race an artificial delay.
- For implementation changes, iterate with subsets if useful, then verify the full suite. Documentation-only changes use the done gate above.
- Never launch system dialogs or native popup paths automatically. If setup, login, permission or a native check needs attended interaction, ask the user directly and report the unresolved acceptance instead of opening it.
- Before a full browser suite in a restricted environment, run one existing test per browser engine with `--max-failures=1` to verify startup. Stop immediately if browser launch is denied or crashes; do not let the suite repeatedly launch failing browsers and flood the desktop with crash dialogs. Report the environment block and leave the full gate incomplete until it is resolved. Do not rerun browser checks after the user asks to stop them.
- Automated tests must emit no sound on the host. Mute every browser context, including manually created contexts, while preserving native decoding, playback clocks and events. Synthetic capture feeds a media stream, never the speaker destination. Live listening and physical microphone checks require attended learner interaction and are separate from automated tests.
- Tests make no real provider, TTS, source mutation or external delivery calls. Fake those boundaries; never read the operator's `.env` or private recordings into fixtures or logs.
- Never leave a failing test. Diagnose clock, ordering, race and environment causes; do not skip, xfail, delete or weaken the assertion to hide a failure.
- Software checks are not language-content, speech-quality or actual-device acceptance. Keep Safari/headset, iPhone and content gaps explicit.
- Prototype fixtures in Git are not replacement compatibility requirements ([architecture](specs/architecture.md#releases-and-data-compatibility)). Preserve meaningful coverage, and replace obsolete expectations only for an intended spec change. The Python application uses library integrations with focused tests rather than carrying forward numerical-port equivalence.

## Code conventions

- Python passes Ruff and Pyrefly; Vue passes Prettier and TypeScript checking. The executable pre gate enforces the handwritten production module-size boundary and generated-contract drift.
- English only in comments, docstrings, plans and repository documentation. Follow the project's English communication rule. Learning data, language names and voice IDs retain their original language/script as quoted, searchable literals.
- All imports at module top level, none inside functions and none to break cycles; fix the dependency structure.
- No re-export shims when moving symbols. Import shared helpers directly from their owner.
- Comments/docstrings explain a non-obvious why in one or two lines: a hidden constraint, library workaround or invariant. Do not restate the code, argue architectural decisions there, or cite phase/plan names; decisions belong in `specs/`.

## Specs and plans

- `specs/` contains current decisions, their rationale and substantial design constraints that are difficult to infer from code. Code, tests and generated contracts document implemented behavior; usage and operational commands belong in README or `docs/`.
- Keep one authoritative home per decision. Do not retain historical references, superseded sketches, review logs, implementation inventories or completed plans in specs. Git preserves earlier versions.
- The implementation plan contains only remaining work. Delete completed entries and plans; retain any lasting decision in its focused spec. Correct contradictory text in the same commit and keep acceptance gaps concise.

## Model integration

- When local AI preparation is used, Codex uses the normal user home and Phrase Gym repository as its working directory, as requested by the learner. Never copy login tokens. Existing private attempt files stay ignored under `.phrase-gym-coach/`; retain subscription-only login, bounded execution and an allowlisted child environment. Deny repository `.env` reads and keep application stores/recordings outside the permitted model workspace. Normal-home instructions/skills may load; do not claim customization isolation. The first MVP imports externally prepared content and needs no internal LLM adapter.

Follow the [standing local English demo authorization](specs/plan/implementation-plan.md#standing-authorization-for-the-local-english-demo): routine demo runs and continuations require no separate budget, temporary authentication workspace or login approval. Existing runtime bounds govern prototype runs; they do not require reproducing its distributed infrastructure. Preserve learner data and recordings during replacement.

When changing prompts, provider payloads or interpretation of real model answers, mocks alone cannot establish model behavior. Plan representative fixtures and real-provider measurements before execution; inspect availability and every concrete result. Record semantic review separately from automated conformance. A fresh reviewer is required for those model-result measurements, not for ordinary refactors. Do not create learner effectiveness experiments or unrequested general review loops.

## Git, secrets and deployment

- Commit authorized work directly to `main` and push it to `origin/main` after the applicable local gate passes, then verify CI as required by the done gate. This is standing authorization; do not ask for separate push approval unless the user explicitly requests local-only work. Do not create branches or PRs by default, and never force-push.
- Publishing releases and deployment still require the user's explicit request for that action.
- Keep `.env`, provider keys, learner data and recordings ignored, private and server-side. Never bundle or print secrets.
- Source repositories, vaults, echo-words, legacy trainers and Anki stay read-only unless separately authorized. Real deployment must be reproducible from committed code and must verify its actual health/access boundary; no one-off server edits.
