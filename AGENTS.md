# Phrase Gym — agent instructions

Read [specs/START-HERE.md](specs/START-HERE.md) and [remaining work](specs/plan/implementation-plan.md). Specs retain current decisions and hard-to-infer design constraints; code and tests describe implemented behavior. Server and Coach are migrating to Rust under that plan; retain the Vue client in `frontend/`. The existing Python scaffold and its commands remain in use until the tested Rust replacement permits their removal.

`CLAUDE.md` links to this file; keep one instruction source.

## Architecture and file structure

- Keep architecture under active review in every batch. Each module and service has one coherent responsibility. Split unrelated workflows before adding more methods; do not grow a catch-all service or application component.
- Aim for no more than 250 lines of handwritten production code per file; review and split before 300. Generated contracts and migrations may be longer. A shorter file does not justify circular dependencies, meaningless wrappers or fragmented invariants.
- Dependencies point inward: transport and UI adapters use contracts/domain services; domain services never import HTTP routers, application startup or UI modules. Current Python contracts live in `src/phrase_gym/contracts/`; the Rust workspace must provide shared contracts at the same architectural boundary. Construct services at the application boundary.
- Keep validation, publication transactions and library reads separate. One workflow owns each transaction; helpers receive that transaction instead of opening hidden writes. Keep all package publication atomic.
- Vue views render state. Route orchestration, preferences and audio effects belong in focused composables/adapters; the practice reducer stays pure. Generated API types are the client contract.
- When moving a symbol, update all importers and tests. No re-export compatibility shims. Shared helpers are preferable to duplicated logic.

## Commands and environment

| Task | Command |
|---|---|
| Lint, format, type check and hygiene | `uv run inv pre` |
| Full Python, client and browser suite | `uv run inv test` |
| Python iteration | `uv run pytest tests/` |
| Client iteration | `npm --prefix frontend test` |
| Browser iteration | `npm --prefix frontend run test:browser` |
| Build client and Python distributions | `uv run inv app-build` |
| Refresh OpenAPI and generated client types | `uv run inv api-types` |
| Available tasks | `uv run inv --list` |
| Rust format, Clippy, architecture, client format and Rust contract drift | `cargo xtask pre` |
| Rust tests, client typecheck/unit/build and both browser engines | `cargo xtask test` |
| Refresh the Rust OpenAPI and its generated TypeScript fixture | `cargo xtask api-types` |
| Build the client and release Rust binaries | `cargo xtask app-build` |
| Run the Rust server on loopback with a synthetic data directory | `cargo xtask run` |
| Re-record scheduler and legacy-schema fixtures from the Python implementation | `uv run python -m tests.preservation.export` |

Use `uv sync --frozen`, `npm --prefix frontend ci`, and install the browser binaries from `frontend/` with `npx playwright install chromium webkit`. Python is 3.12+; use Node 22.12+ on an LTS release (`.nvmrc`). Rust uses the exact toolchain pinned in `rust-toolchain.toml` through rustup. The pyrefly hook uses `.venv/bin/python`; the Rust hook runs `cargo xtask pre` and needs that toolchain plus Node. Adding/changing Python dependencies means updating `uv.lock`; client dependencies update the npm lockfile; Rust dependencies update the committed `Cargo.lock`, disable default features and list the features they need. Missing dependencies or browsers are environment failures to fix, not reasons to skip suites.

During the transition `uv run inv pre` and `uv run inv test` remain the authoritative gate: the first runs `cargo xtask pre` as a hook and the second runs the Rust tests. `cargo xtask` replaces them only at the cutover described in the plan.

**Never call Ruff directly.** `inv pre` runs the project's configured Ruff, formatter, pyrefly and hygiene hooks and matches CI. An editor lint panel does not replace it.

## Done gate

Choose the gate from the complete change, including staged and unstaged files.

**Documentation-only changes:** when the change contains only prose in specs, plans, README, docs or agent instructions, do not run `uv run inv pre`, Python checks, application builds or Python/client/browser suites. Run `git diff --check` and `git diff --cached --check`, inspect the full diff, verify changed local links/anchors and check consistency with related decisions and plans. Report these documentation checks without claiming application tests passed. Documentation describing future behavior does not require implementing or testing that behavior.

**Implementation or mixed changes:** changes to production code, tests, runtime prompts, generated contracts, migrations, dependencies, executable scripts, build configuration, hooks or CI require the full gate, even when accompanied by specs. Before describing such a milestone as done, ready, fixed, landed or green, run these in order and inspect their complete results:

1. `uv run inv pre`: every hook passes, including the Rust hook, and pyrefly reports zero errors.
2. `uv run inv test`: the full Python, Rust, client and browser suites pass; frontend checks must run rather than skip.

Run `inv pre` after each discrete implementation batch; use the documentation-only gate for prose-only batches. If a hook rewrites files, rerun until it converges. A previous run or a narrow test subset does not satisfy the implementation gate. Never bypass commit hooks with `--no-verify`.

Fix findings even when they appear pre-existing. Deferral requires evidence that the same failure exists on the committed baseline and the user's agreement in the current turn. Never silently report a failing tree as passing. If execution or approval infrastructure blocks a check, describe that block explicitly and do not claim the gate passed.

## Tests

- New modules/functions ship meaningful behavioral coverage in the same commit. Keep tests with the responsibility they verify; do not grow one catch-all test file.
- Behavior that only exists in the browser needs a real-browser test against the real server and built client. Assert provisional/error states deterministically; do not race an artificial delay.
- For implementation changes, iterate with subsets if useful, then verify the full suite. Documentation-only changes use the done gate above.
- Tests make no real provider, TTS, source mutation or external delivery calls. Fake those boundaries; never read the operator's `.env` or private recordings into fixtures or logs.
- Never leave a failing test. Diagnose clock, ordering, race and environment causes; do not skip, xfail, delete or weaken the assertion to hide a failure.
- Software checks are not language-content, speech-quality or actual-device acceptance. Keep Safari/headset, iPhone and content gaps explicit.
- `fixtures/preservation/` records legacy behavior for the Rust replacement: synthetic HTTP scenarios, scheduler sequences and legacy schemas. Re-record them only for an intended legacy change. Every legacy test module keeps an entry in `fixtures/preservation/successors.toml`; add, move or delete entries with the module, and delete a legacy harness only when its entries are covered or superseded.

## Code conventions

- While Python remains, use Python 3.12+, `src` layout, hatchling packaging and the existing uv, Click, pytest, Invoke and CI conventions. Add Rust tooling during migration and replace Python tooling at the tested cutover described in the plan.
- Rust code passes rustfmt and Clippy pedantic with warnings denied; documentation lints are relaxed because comments default to none. The `xtask` architecture check enforces crate dependency direction, explicit dependency features, a Server dependency graph free of GUI, provider and model crates, and the module-size boundary.
- English only in comments, docstrings, plans and repository documentation. Follow the project's English communication rule. Learning data, language names and voice IDs retain their original language/script as quoted, searchable literals.
- All imports at module top level. No in-function lazy imports and none to break cycles; fix the dependency structure. An optional heavy provider dependency needs an explicit documented exception when actually integrated.
- No `from __future__ import annotations`.
- No re-export shims when moving symbols. Import shared helpers directly from their owner.
- Comments/docstrings explain a non-obvious why in one or two lines: a hidden constraint, library workaround or invariant. Do not restate the code, argue architectural decisions there, or cite phase/plan names; decisions belong in `specs/`.

## Specs and plans

- `specs/` contains current decisions, their rationale and substantial design constraints that are difficult to infer from code. Code, tests and generated contracts document implemented behavior; usage and operational commands belong in README or `docs/`.
- Keep one authoritative home per decision. Do not retain historical references, superseded sketches, review logs, implementation inventories or completed plans in specs. Git preserves earlier versions.
- The implementation plan contains only remaining work. Delete completed entries and plans; retain any lasting decision in its focused spec. Correct contradictory text in the same commit and keep acceptance gaps concise.

## Model integration

When changing prompts, provider payloads or interpretation of real model answers, mocks alone cannot establish model behavior. Plan representative fixtures and real-provider measurements before spending budget; inspect availability and every concrete result. Record semantic review separately from automated conformance. A fresh reviewer is required for those model-result measurements, not for ordinary refactors. Do not create learner effectiveness experiments or unrequested general review loops.

## Git, secrets and deployment

- Commit authorized work directly to `main`. Do not create branches or PRs by default.
- Never push, publish or deploy without the user's explicit request for that action. A local commit does not authorize any of them.
- Keep `.env`, provider keys, learner data and recordings ignored, private and server-side. Never bundle or print secrets.
- Source repositories, vaults, echo-words, legacy trainers and Anki stay read-only unless separately authorized. Real deployment must be reproducible from committed code and must verify its actual health/access boundary; no one-off server edits.
