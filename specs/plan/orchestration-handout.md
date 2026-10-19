# Orchestration handout: continuing the Rust migration

This handout lets a new Astra session take over orchestration of [the implementation plan](implementation-plan.md) and [the Coach desktop plan](coach-desktop.md) with no access to the previous session's memory. Read it fully before acting. Written 2026-10-18.

## 1. Your role (the user's instruction)

The user asked: implement `specs/plan/implementation-plan.md`, working **as orchestrator only**:

1. For each stage, run the implementation in a **fresh session** (GPT-6.1 / Sol 6.1; Agent tool parameters `model: "gpt-6.1-sol"`, `reasoning_effort: "medium"`).
2. Then run a review in a **fresh session**.
3. Decide which findings to pass on, and run the fixes in a **fresh session**.
4. Review again in a fresh session.
5. If reviews do not converge, analyse the cause and go to the user with a **specific suggestion** of what to simplify or remove from the requirements or invariants to improve convergence. Then **wait for confirmation**.

Do not write production code, tests or specs yourself. Your own work is limited to:

- writing prompts;
- reading reports;
- triaging findings;
- light verification such as `git log`, `git status`, `gh run view` and small diffs;
- talking to the user.

## 2. Binding user decisions so far

These override older plan or spec text wherever they conflict. Most of them are already written into specs and plans; restate them in every relevant prompt anyway, because agents start cold.

1. **No backward compatibility (2026-10-17).** "We do not have any installations of Python versions … I need absolutely nothing left from current Python version, no backward compatibility, we have absolutely nothing installed from Python version."
   - The Rust version is the first version users install.
   - No migration from Alembic schemas.
   - No migration of old job states, preferences or revisions.
   - No compatibility with Python-era data, receipts or queued payloads, server or browser.
   - No versioned contracts kept for old clients: an API change ships with the matching client change in the same commit.
   - This is recorded in `specs/architecture.md` under "Releases and data compatibility".
2. **Reuse the Vue client; never rewrite it (2026-10-17).** "We spent a lot of time making mic working in frontend - I expect we just reuse current frontend with refactoring to new requirements - no need to rewrite it from ground up. But still no compatibility with old data in browser DB etc."
   - Keep, refactoring only where contracts change: microphone capture, audio session, the take journal with recovery and retries, playback, and their real-browser tests.
   - Remove only code that reads, migrates or retires data left by older client versions: old journal key formats, the old offline service worker and caches, historical rating flags, and the legacy browser harness.
   - Never pass on a review finding whose fix is "rewrite the client".
3. **Not compatibility, so these stay:**
   - the FSRS reference fixtures (the spec defines scheduler equivalence by them);
   - preservation scenarios that encode spec rules: same receipt on identical retry, conflict on a changed payload, atomic rollback, byte retention.
4. **GitHub CI and real CLIs (2026-10-16).** "Push to main if you need github CI to test coach on windows. And you always can test coach on local macos. Local sessions already have codex and claude CLI auth, so you just run them without explicit auth like news-recap do."
   - Pushing `main` is authorized **for Coach Windows CI only**. No tags, releases or force-push.
   - A push also runs the docs workflow; that is the user's normal flow. `pip_publish` publishes only for a `vX.Y.Z` tag.
   - Real `claude`/`codex` runs are allowed locally on synthetic inputs only. Keep them bounded (about 15 per CLI per spike), and never put them in automated tests.
5. **Headless CLI knowledge must not be re-debugged.** The user spent a long time on this in `../news-recap`.
   - The proven invocation contract, its evidence, the pitfalls and the news-recap source pointers live in `docs/coach-desktop.md`. Decisions are in `specs/decision-external-preparation-agent.md`, and the code is the authority for flags (`crates/phrase-gym-coach/src/adapter/`).
   - Key points: the prompt goes in a file with a short bootstrap argument, never on stdin. Claude runs `-p … -- "<bootstrap>"` and never `--bare`. Codex runs `exec --skip-git-repo-check` with a permission profile, not `--sandbox`. The environment is allowlisted with API keys stripped. The working directory is a fresh per-attempt folder. The answer is read from output, and empty output is a failure.
6. **Approved behavior: an outside pause ends the comparison (2026-10-17).** Applies when the system, a headset or a media key pauses playback during comparison.
   - Playback stops quietly, the main button returns to Record, and both replays stay available.
   - Implement it in **stage 6** (client batch): a one-line decision in `specs/decision-recording-and-comparison.md`, the client change, and a real-browser test that pauses the media element from outside.
   - A real-device check joins batch 8.
7. **Pending user decision: Coach's own Codex home.**
   - Recommended, and already implemented in code, plan and docs.
   - The user still has to sign in once. As of 2026-10-18 the folder exists but nothing is signed in:

     ```
     D="$HOME/Library/Application Support/io.github.andgineer.Phrase-Gym-Coach/codex-home"; mkdir -p "$D" && chmod 700 "$D" && CODEX_HOME="$D" codex login
     ```

   - After that, run the confirming probes in `docs/coach-desktop.md` (manual checklist step 9).
   - Only macOS real-Codex adapter work (Coach C4) waits on this.
8. **The user's global rules for talking to them** (from their `~/.claude/CLAUDE.md`):
   - Always answer in English.
   - Never hand over a decision without a recommendation. State the facts in plain sentences, then the options (each explained by what the user sees), then a comparison table of concrete situations and costs, then **one recommendation**, phrased so the user can answer "yes". No function names or file paths in the decision part.
   - Never publish claude.ai artifacts.
   - Documents the user asks for go in the repository, not temp files.
   - Specs hold decisions and rationale only: no signatures, field names or change history. Plans may hold full implementation detail. Comments default to none, or 1–2 lines explaining a non-obvious why.

## 3. Where things stand (main at `af48263`, origin at `2ffd4a0`)

| Stage | Commits | Status |
|---|---|---|
| 1 Rust tooling and preservation fixtures | `df38eb1`, `9160179`, `b050dc9` | Converged after 2 review rounds |
| Coach C1 desktop boundary | `3518924`, `27ebda9`, `9e5a0c5`, `2960f81`, `af268fb`, `2ffd4a0` | Converged after 3 rounds; CI green on Windows, Intel and Apple Silicon (run 39047856727). Open gaps are listed in `specs/plan/coach-desktop.md` C1 and the manual checklist in `docs/coach-desktop.md`. |
| 2a no-compatibility docs, Rust-native schema, request boundary, content, library, reference media | `c7ea8e1`, `827f7f6`, `d843b58`, `fa24df3`, `7da2156`, `55c87e0` | Converged after 2 rounds |
| 2b part 1: sessions, events, receipts, repairs, current planning, learner recordings, settings, cue reports; every endpoint the client calls | `af48263` | **Implemented, not yet reviewed.** The implementer was interrupted by a usage limit while its final gate was running, so the gate on `af48263` is unconfirmed. |

Local `main` is ahead of `origin/main`. Do not push except as allowed in section 2, item 4. Everything is committed; the tree was clean at handoff.

## 4. Remaining order of work

The user's current priority is a small usable macOS preview. Execute the [bounded preview stages and exits](implementation-plan.md#bounded-preview-stages) before returning to the full-release backlog below. That plan is the single home for preview scope, acceptance and the freeze on broader recovery work; an unresolved demonstrated risk of learner-data loss still blocks the preview. Keep the fresh-session implementation/review/fix workflow, with reviews bounded to the delivered stage and concrete defects.

The remaining full-release work is:

1. **Stage 3:** profiles, revised content and target lifecycle.
2. **Stage 4:** durable inbox, fenced jobs and publication APIs, including the Coach summary endpoint.
3. **Stage 5:** planning, evidence and progress.
4. **Coach C2–C4** (dependencies are in `specs/plan/coach-desktop.md`):
   - C2 is the headless runner against the real Rust Server.
   - C3 needs the relevant stage 3–4 contracts and APIs.
   - C4 needs the revised package format and real adapters.
   - Windows adapter-dependent work waits for the Windows real-CLI probes; it does not block the macOS preview.
5. **Stage 6:** remaining client integration plus Coach C5 and C6, plus decision 6 (outside pause).
6. **Stage 7:** the cutover that removes every trace of Python.
7. **Batches 8–9** need separate user authorization: provider budget, source projects, devices, deployment. Stop before them and report.

**Split large stages** into parts with their own review cycle, as was done for stage 2 (2a, 2b part 1, 2b part 2). Smaller diffs review better and survive usage limits. Record a split in the plan, either through the implementer or in the next implementer's prompt.

## 5. How to run each step

### Mechanics

- Select checks from the complete change under [AGENTS.md's done gate](../../AGENTS.md#done-gate). The implementation/review/fix prompt templates below describe implementation batches; prose-only planning stages use documentation checks and do not run application gates. A fresh review does not by itself change a prose-only stage into an implementation batch.
- **Agent tool:** implementation sessions use `model: "gpt-6.1-sol"`, `reasoning_effort: "medium"`. Implementation, review, fixes and re-review each start in a fresh session, with the root acting only as orchestrator. Run in the background, **one session at a time**. Gates use fixed ports for the browser suites, so two concurrent gate runs collide.
  - Reviewers must work in a fresh `git clone` under the scratchpad, never in the main tree.
  - Implementers and fixers work in the main tree.
- **Never read an agent's output JSONL file.** It overflows context. To check what an agent did (for example after a harness "security warning"), filter its tool calls with a short Python script. A previous warning turned out to be the agent deleting its own scratch folder.
- **Usage limits interrupt agents often.** After a reset:
  1. check `git log` and `git status`;
  2. resume the same agent with `SendMessage` ("the usage limit has reset; continue from where you stopped; last commit X");
  3. do not start a duplicate.

  Tell every implementer to commit coherent green progress in several commits, and to keep its final report short.
- **If an agent hands back before CI or a gate finishes**, verify it yourself (`gh run view <id> --json jobs`), or resume it.
- **Environment:**
  - Node 22 via nvm: `source ~/.nvm/nvm.sh && nvm use`, with `.nvmrc`. The default PATH Node 20 fails the Rust hook by design.
  - The Rust toolchain is pinned in `rust-toolchain.toml`.
  - The pre-commit hook is **not installed** in this checkout, so agents must run `uv run inv pre` themselves before committing.
  - The transition implementation gate is `uv run inv pre`, then `uv run inv test`; these already include the Rust checks/tests. Run `cargo xtask coach-smoke` for packaged Coach changes. Do not duplicate the included Rust gate without a new change, failure or unresolved concern.
- **Flaky tests must be diagnosed and fixed** (AGENTS.md); never retried, skipped or padded with timeouts. Three were found and fixed so far:
  - host Now Playing pauses → test playback is muted;
  - a Resume click landing during a layout shift;
  - a WebKit request-event race → wait for the request instead.

### Implementation prompt (essential parts)

- **Repo and starting state:** path, branch `main`, HEAD hash, and that the tree is clean. Say the agent has no prior context and is one step of an orchestrated pipeline.
- **Mandatory reading:**
  - AGENTS.md (binding rules, done gate);
  - `specs/START-HERE.md` and `specs/architecture.md`;
  - the whole implementation plan;
  - the specs the stage links;
  - for Coach work, `specs/plan/coach-desktop.md` and `docs/coach-desktop.md`;
  - the existing Rust crates and the Python code being ported.
- **The binding user decisions** from section 2 that apply.
- **The exact scope:** which plan bullets (and which part, if split), plus what is explicitly *not* in scope (later stages).
- **Plan upkeep:** delete completed plan entries and keep lasting decisions in their focused spec, following the spec rules.
- **Ground rules:**
  - no subagents, no pushes (except the Coach CI case), no real provider calls, no private data, no other repositories (`../news-recap` and `../ibkr-porez-rs` are read-only);
  - comments default to none;
  - the 250/300-line rule;
  - meaningful tests;
  - commit often;
  - run `inv pre` before each commit;
  - the done gate before reporting.
- **Final report:** commits, what each item has, gate results, deviations and risks, plan/spec edits. Keep it compact.

### Review prompt (essential parts)

- **Isolation:** independent and read-only. Work in a fresh clone in the scratchpad. No commits, pushes or subagents, and no real provider calls (`--help`/`--version` are fine).
- **Requirements:** the stage text **as it stood before the stage**, e.g. `git show <base>:specs/plan/implementation-plan.md`, because implementers delete completed entries. Give the commit range, the binding user decisions, and any scope notes or declared deviations to judge.
- **Checks:**
  1. requirement coverage, delivered/partial/missing with evidence;
  2. correctness, with a **concrete failure scenario** for each finding and no speculation;
  3. rules: architecture, module size, dependency direction, meaningful tests (try mutations), documentation placement;
  4. gate honesty: run the full gate on the committed tree and confirm it passes without rewrites; for Coach, also check the CI logs.
- **Exclusions:** no style preferences, "could also add" ideas, or client rewrites.
- **Output:**
  - gate results;
  - a coverage table;
  - findings with ID, severity (blocker/major/minor), `file:line`, the exact rule quoted, the failure scenario and a minimal fix.

### Fix prompt (essential parts)

- Starting state and the stage context.
- The accepted findings, pasted in full with locations, scenarios and the expected fix. Note any finding the orchestrator narrowed or declined, and why.
- Ask the fixer to:
  - fix exactly these findings;
  - prove each new test with the mutation it guards against;
  - run the full gate;
  - commit;
  - for Coach, push and confirm CI.
- A short report per finding.

### Re-review prompt (targeted)

- Verify each previous finding: resolved, not resolved, or resolved with a new problem. Try to break each fix.
- Review the fix diff for regressions.
- Do a final pass over the whole stage for **blocker/major only**; minors count only if the fix commits introduced them.
- State effects the orchestrator already accepted, so they are not re-reported.
- Run the full gate.

### Triage rules used so far

- **Accept:** blockers and majors with a concrete scenario; cheap, real minors (tests that cannot fail, overclaiming docs, comment-rule violations).
- **Reject or narrow:** findings that conflict with a user decision; style; "could also add"; anything requiring a client rewrite.
- **Deviations:** accept an implementer deviation from the plan when it is backed by evidence. Examples: the Codex permission profile instead of `--sandbox workspace-write`, and SQLite `-shm` bytes changing on a refused database.
- **Product questions found by agents** (such as the outside-pause behavior): bring them to the user in the decision format. Do not let agents decide them.

### Convergence

- **Converged** means a review round with no new blocker or major.
- So far stages have converged in 2–3 rounds, with shrinking findings: C1 went from 4 majors, to 2, to 0.
- **If a third round still yields new majors of the same class:**
  1. stop;
  2. analyse why (requirement too broad, invariant too strict, reviewer and implementer reading a rule differently);
  3. propose to the user one concrete simplification or removal, in the decision format;
  4. wait.

## 6. Known open items to carry forward

- **Coach C1 gaps** (in `specs/plan/coach-desktop.md` and the manual checklist in `docs/coach-desktop.md`):
  - VoiceOver and Narrator checks;
  - the oldest supported OS versions;
  - real-CLI probes on Windows, which block Windows adapter-dependent work;
  - the Coach-home Codex probe (section 2, item 7).
- `preparation-lifecycle.spec.ts` drives the Python preparation harness. It has no Rust successor before stages 4 and 6.
- `fixtures/preservation/successors.toml` maps every legacy test module to its successor. Legacy harnesses may be deleted only when their entries are covered or superseded, and the cutover (stage 7) deletes the record itself.
- The Python gate stays authoritative until stage 7, which switches the done gate to `cargo xtask` in the same change as AGENTS.md and CI.
