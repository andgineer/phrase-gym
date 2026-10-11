# Brief for the next session

Read [START-HERE](../START-HERE.md), the [drilling decision](../decision-targeted-drilling.md), the [recording decision](../decision-recording-and-comparison.md), and the [interaction design](../design/interaction-design.md) before doing anything from an older implementation plan.

The original handoff closed a design/planning conversation for continuation with a different model. On 2026-10-11 the user resumed **prototype and functionality discussion**, requiring all decisions to be preserved in `specs/`. **No application implementation has been requested.** Follow the user's actual request and the [current discussion record](../design/interaction-design.md#resumed-prototype-discussion-2026-10-11). Do not resume the archived implementation automatically.

The latest request adds [academic evidence](../research-answer-card-and-scheduling.md) and a [recommended answer card/scheduler](../design/answer-card-and-review-policy.md): Not yet / With effort / Recalled, separate extra practice and FSRS with a 0.90 default retention target. These specifics are recommendations, not yet accepted. **Accepted correction:** the user will not validate their learning or run experiments. Choose a scientifically grounded method with usable defaults; no validation UI, special assessment session or calibration task. Normal ratings drive automatic scheduling; engineering checks belong to the implementer.

## Product to build when implementation is requested

Phrase Gym drills specific words in phrases, fixed expressions and bounded prepared dialogues toward ready use. One Start session entry leads to drills of material expected to have been encountered in its source. Do not create separate Learn new material/Rehearse home modes or force an introduction for every new database item. The drill cues the intended expression with sufficient meaningful material and asks for the whole phrase. A visible explanation action must show the complete phrase and help in the learner's language when it is forgotten or not understood; normal drills remain in the target language. Track that help/exposure. Self-comparison, repair, useful cue variation, alternatives/ambiguity handling and one schedule per target remain agreed.

Accepted cue-preparation clarification: the LLM prepares alternate hidden-word/chunk versions when a phrase is added to the app database, where the phrase supports them. Store checked variants with the same phrase and retain presentation history across sessions so a later session can show a different version. A phrase with only one reliable cue may reuse it; no fixed variant count or runtime random masking is required.

Accepted home/navigation addition: a searchable lesson dropdown defaults to **All lessons**, using spaced repetition; typing `rap` filters lesson titles to matches such as Building rapport. Separate phrase lookup supports meaning/intent terms absent from the phrase (for example `greetings`) and opens Show & explain directly, without treating lookup as a review. Show & explain links to its lesson's complete phrase list, with a Start lesson drill action there. Selected-lesson due-only versus all-phrase policy and search implementation remain open. Read the [navigation/search decision](../decision-session-navigation-and-search.md); preserve one target/schedule and exposure history across these routes.

On first launch with an empty database, ask for native language and persist it in app settings for the explanation card. Reuse it on later launches; target-language metadata remains separate. Russian is selected for the current drill/explanation sketch, not the app's universal default. Current visual proposals and accepted behaviour are separated in [interaction design](../design/interaction-design.md).

The recording decision adds optional session recording. With recording enabled, automatically play the learner's take, then reveal/play the model, then offer self-check and rerecording. With recording off, allow silent practice using Reveal & compare and manually requested model audio. Manual Stop recording is the accepted default and the only stopping method in the first version. Choosing manual versus automatic stop belongs in later app settings; the selector and automatic stopping are deferred beyond the first version.

The user also selected automatic database retention of **all recordings from the first version**, including retries. Persist audio bytes and linked target/attempt/cue/timestamp/support metadata; no manual Save step, replacing previous takes or automatic expiry. The proposed SQLite design uses BLOBs for learner audio. A historical progress-comparison UI is deferred; keep immediate self/model replay. This replaces earlier selective-saving and media-file proposals for learner takes.

Accepted addition: Show & explain permits practice recording while the complete phrase remains visible through capture and learner/model playback. Use the same microphone/Stop recording controls, session recording switch, automatic comparison and retention policy. Mark these as model-supported practice, not independent recall; retain exposure when returning to the drill. Do not implement the explanation card as read/listen-only.

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
