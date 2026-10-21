# Fresh-session orchestration handout

Read [AGENTS.md](../../AGENTS.md), [START-HERE](../START-HERE.md) and the whole [implementation plan](implementation-plan.md). The implementation plan is the single home for remaining stages, their order, scope and acceptance. Follow its links to focused decisions and operational documentation; do not duplicate its task lists here.

## Role and workflow

The root session orchestrates only: write prompts, read reports, triage findings, perform light verification and talk to the user. Delegate production code, tests and spec/plan edits.

Take the first remaining stage from the implementation plan. Select a bounded coherent slice under its acceptance criteria, preserving the existing implementation rather than rebuilding completed work. For each slice:

1. Delegate implementation to a fresh session.
2. Delegate independent review to another fresh session.
3. Triage findings and return accepted fixes to the original implementer.
4. Ask the independent reviewer for targeted re-review of substantive findings and their regressions.

Use one implementer and one independent reviewer per coherent deliverable, reusing them for fixes and targeted re-review. Use GPT-6.1 / Sol 6.1 medium: `model: "gpt-6.1-sol"`, `reasoning_effort: "medium"`, `fork_turns: "none"`. Supply enough context for agents with no prior conversation. Do not delegate further subagents. Implementers and fixers work on `main`; reviewers use a fresh scratch clone and do not modify the main tree.

Run one session at a time; avoid competing writes and concurrent gates. After interruption, inspect agent status, Git state and outstanding checks, then resume existing work instead of duplicating it. Do not claim a running gate passed or substitute a different model after a usage limit.

## Prompts and review

State the observed branch, HEAD and working-tree changes, exact plan slice, acceptance criteria, required reading and applicable permissions. Preserve unrelated changes. Give reviewers the delivered commit range and requirements as they stood before implementation, obtainable with `git show <base>:specs/plan/implementation-plan.md`.

Require concrete findings with severity, file/line, governing rule, failure scenario and minimal fix. Review coverage, correctness, architecture and gate honesty; reject speculative additions or scope expansion. Send accepted findings in full to the fixer, with any narrowing explained. Re-review fixes and regressions, then the stage for blockers and majors; unrelated minors count only if introduced by the fixes.

Completion requires an independent review with no unresolved blocker or major; ordinary fixes do not require a chain of fresh implementers and reviewers. Fresh semantic review of actual model-result measurements remains required by AGENTS.md. If reviews fail to converge, stop, explain the cause and propose one concrete requirement or invariant to simplify or remove. **Wait for the user's confirmation** before changing it. Bring product choices to the user with visible consequences and one recommendation.

## Completion and boundaries

Every demo delivery states the observable new benefit to the learner and proves that workflow live with matching runnable components. Testing, refactoring, CI repairs or other internal checkpoints alone are not a demo. Preserve the full MVP scope while making each bounded demo a useful increment; report unverified content, listening and device acceptance explicitly.

Delegate verification under the complete-change [done gate](../../AGENTS.md#done-gate), including the full prescribed gates for implementation/mixed changes and documentation checks for prose-only changes. Inspect complete results before reporting completion; keep model/content, native and actual-device acceptance separate from software conformance.

Require implementers to delete completed stage entries from the implementation plan, preserve unresolved acceptance and keep lasting decisions in focused specs. Keep this handout stable and compact. Do not add completed-stage logs, implementation inventories, commit histories, test-count audits, incident narratives or temporary runtime details to specs; Git preserves history.

Commit authorized work directly to `main`; do not push, publish or deploy without explicit authorization. Follow AGENTS.md and the focused specs for provider measurements, privacy, read-only sources and effective permissions. Report actual blocks rather than bypassing approvals or promising automatic quota recovery. Finish with a concise report of commits, checks, unresolved acceptance and material deviations.
