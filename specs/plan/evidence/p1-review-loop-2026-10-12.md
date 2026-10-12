# P1 fresh-session review loop — 2026-10-12

The user requested independent Sol 6.1 review/fix iterations, stopping before a fourth iteration if actionable defects remain. Reviews use fresh local Codex chats with no inherited discussion. The coordinating chat assesses findings, fixes necessary P1 defects and runs regression checks before the next review. No production/P2 work or acceptance gate is implied.

## Iteration 1

Reviewer: **P1 review — Sol 6.1 — iteration 1**, chat `01a12ea1-e0b0-7413-a534-4ff088056109`, model `gpt-6.1-sol`, high reasoning. Read-only review reproduced nine defects despite all existing tests passing. All nine were accepted for correction:

| Finding | Correction |
|---|---|
| A new capture discarded an unsaved take after a write failure | Block new capture until the pending take is stored; keep the recovery banner visible across target navigation |
| A stale browsing tab overwrote committed history on a preference change | Separate preference and history writes; update individual shared records inside IndexedDB read/write transactions; only the lease owner writes session checkpoints |
| Search/list answers were visible without retained exposure | Journal displayed search/list phrases and classify immediate follow-up recall as supported |
| Undo left an inserted retry in the queue and regrading inserted another | Restore the saved queue together with cursor, completed items and repair counters |
| Explanation practice replaced the original response replay; Undo lost replay | Keep the original take ID and select it for ungraded original attempts, including corrected attempts |
| Dialogue items omitted early-practice eligibility | Apply the same due/receipt check used by ordinary sessions |
| Delayed reference playback continued after Pause | Invalidate asynchronous comparison on navigation/stop, guard post-fetch playback, and resume completed original responses at self-check |
| Generated imports replaced actor audio | Store actor and generated imports separately and select actor audio first, while retaining legacy imported records |
| Situations supplied masked wording | Rewrite situations without the hidden chunk; keep natural task context and prepared cue identity; fixture revision `.2` |

Focused regressions are in `prototypes/tests/review.cjs`; policy/fixture regressions were added to `core.test.mjs`. Browser tests use synthetic microphone input and do not establish device or content acceptance. Original baseline: 9 policy tests and 10 browser walkthroughs passed. Post-fix validation: **11 policy/fixture tests, 10 original browser walkthroughs and 7 focused browser regressions passed**. The focused harness blocks service workers so it can intercept delayed fetches; the original browser suite separately verifies offline/service-worker behavior. An initial parallel run of the focused harness timed out during onboarding and request interception; the isolated deterministic run above passed.

## Iteration 2

Reviewer: **P1 review — Sol 6.1 — iteration 2**, chat `01a12eaf-34df-70a1-924d-21eaf536dfd4`, model `gpt-6.1-sol`, high reasoning. Independently reran all first-round tests and found six further defects; all were accepted:

| Finding | Correction |
|---|---|
| Undo during the next capture left the microphone live without its stop control | Cancel capture/comparison before restoration, retaining interrupted bytes and refreshing the original attempt |
| Result-journal failure committed a receipt and finalized the in-memory attempt without a persisted result | Build a draft result/checkpoint and atomically commit history, result event and session in one IndexedDB transaction; use the same transaction boundary for Undo; failed writes leave the original state retryable |
| Explanation replay could play through an active microphone | Disable replay controls during pending/active capture and automatic comparison; guard handlers as well |
| Home's recording switch could not enable recording after Finish | Update the same state that Home renders for the next session |
| Ordinary reveal/automatic comparison exposure was lost between sessions | Journal answer exposure after snapshotting the original response; subsequent immediate attempts retain support, including retries; original grading remains valid |
| Probe Stop did not cancel a pending reference fetch | Guard delayed manual/automatic/inspection playback with a cancellation generation |

Added seven browser scenarios covering these six findings, including both manual and automatic answer exposure and write faults in each of the three result stores. Post-fix validation: **11 policy/fixture tests, 10 original browser walkthroughs and all 14 focused browser scenarios passed**. A focused test initially used a silent Reveal action while its next session still had recording enabled; its setup was corrected to switch recording off explicitly.

## Iteration 3 and pause

Reviewer: **P1 review — Sol 6.1 — iteration 3**, chat `01a12eb9-32fc-7e21-9663-8c53ca174c5c`, model `gpt-6.1-sol`, high reasoning. All existing checks passed; two further defects were reproduced and accepted:

| Finding | Correction |
|---|---|
| A delayed manual reference began playing after microphone capture started (both surfaces) | Starting capture invalidates pending playback; handlers recheck after loading; the shared audio player rejects playback during pending/active capture |
| Stop during learner replay revealed the answer without logging exposure | Retain answer exposure on the stop-to-self-check transition while preserving the original response eligibility |

Two focused browser scenarios cover the reported triggers. Final validation: **11 policy/fixture tests, 10 original browser walkthroughs and all 16 focused browser scenarios passed**. Build `0.1.1` uses a new shell-cache version so offline work also receives the reviewed code; the original offline walkthrough is rerun after that version change. No fourth fresh review has been started: all three reviews produced findings (9 → 6 → 2). Known third-round defects were fixed within the third iteration; an independently clean review has not been established.

**Convergence analysis:** tests covered walkthroughs and individual bug fixes, while overlapping actions and alternate exits were handled separately in UI callbacks. Fixing one route left other ways to reach the same state unchecked. The product goals are clear; the implementation plan needs a small shared transition table for cue, permission, capture, pending playback, comparison, self-check, supported practice and pause. Each transition should specify microphone/playback cleanup, original attempt/take identity, answer exposure and the transaction boundary. Test those invariants across both the drill and probe, including delayed requests and storage failure, before another independent code audit. This restructuring is a proposal, not approved implementation work.

The user subsequently questioned further automated reviews for disposable prototypes. Recommendation: stop this automated loop after the current fixes and commit, perform the P1 manual walkthrough, record concrete usability corrections, and agree the production structure before P2. Manual review should assess the visible flow and cue clarity; it does not establish production reliability or replace actual-device evidence. Await user confirmation before another fresh-session loop or the proposed restructuring.

## User follow-up: portable providers and source layout

The user rejects depending on macOS-only speech and asks to reuse echo-words audio engines and llmbroker, with keys supplied in the local `.env`. Samantha is existing disposable fixture provenance, not a required runtime dependency or the production provider. Reconfirmed the local reference: Piper Lessac for English, Piper Thorsten for German, language-matched Edge fallback, Edge Sophie for Serbian with Cyrillic speech input; actor/course clips retain priority. llmbroker's AsyncBroker with Secrets(env_file=...) supplies server-side LLM preparation, separately from speech synthesis. No provider keys or private audio belong in frontend assets or commits; no live LLM call is required for P1's manually authored fixtures.

P1 explicitly requires runnable disposable work under `prototypes/`; `src/` remains the scaffold because P2 was not begun. The user's direct-main commit preference is authorized and should persist for subsequent implementation; do not create an implementation branch by default. Production adapters and normal application code belong in `src/phrase_gym` during the authorized production phase. Provider adoption and architecture details should be carried into that next scope discussion.

## Suggested manual walkthrough

Use `python3 prototypes/serve.py` (or the running loopback server on port 8873) and follow `prototypes/README.md`. Prioritize: home/search → explanation → lesson drill; normal recall versus forgotten/helped practice; recording on/off and learner-first comparison; extra practice/Undo; English/Serbian switching and pause/reload. Report any confusing cue, wording, button placement or unexpected behavior. Do not run special learning assessments.

## Acceptance remains separate

User walkthrough, actual iPhone Safari/home-screen audio/storage checks, trusted private HTTPS access, authentic actor/Serbian audio and native-speaker content confirmation remain pending. P1 is not passed and P2 has not begun. These disclosed dependencies do not count as defects for the convergence loop.
