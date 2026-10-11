# Session decisions and handoff — 2026-10-11

This is a focused decision record, not a verbatim transcript. It preserves the user intent and status needed by a fresh session without reviving rejected directions.

## Scope and learning behaviour established in this conversation

- The user rejected the earlier open-ended AI speaking/role-play option as completely outside scope. The app is for selected words in useful phrases and specific dialogues containing the expressions the learner wants to use automatically.
- Research was requested to inform the drills. The resulting evidence record distinguishes retrieval/spacing findings from unproven product choices; cue rotation and the whole spoken workflow must be evaluated.
- The user objected to blank situation questions demanding one arbitrary phrase. Enough identifying material must be shown for each particular phrase, from whatever meaningful chunks work. The hidden part may change deliberately between repetitions.
- The full expression is introduced before recall. Whole-phrase responses, appropriate equivalents, word/frame slots, ambiguity handling, actual support history and one target-level schedule are recorded in the unchanged drilling decision.
- The app name chosen by the user is **Phrase Gym**. The short repository description is recorded in START-HERE.

## Planning and project location

The user asked for architecture and implementation plans and corrected the assistant: no Phrase Gym prototypes existed; prototyping had to be included as future work. The resulting technical documents propose an architecture and P1–P6 roadmap, rather than approving stack, UI, schedule parameters or hosting by implication.

The user then created `~/projects/phrase-gym` using a Python package scaffold and requested moving the development documents from phrase-shadowing. That first move was not applied. The assistant instead began implementation in temporary staging. The user explicitly stopped it:

> “I want design discussion, written plan and after that I will start in new session with different model”

> “I want discuss disign with you here!!!! after that you will write all our decisions to the plan for new session”

The assistant stopped the prototype server and confirmed that neither project directory had received its implementation changes. Its staging archive is historical and unapproved.

## Design discussed after the stop

The assistant proposed one expression per screen, context above a prominent phrase cue, whole-phrase response, comparison and help controls. It recommended a calm visual direction: warm neutral background, restrained accent, large phrase text and generous spacing. The user did not explicitly choose the visual direction, colours or typography. No rendered mockup became an approved design.

The user added the important missing interaction:

> “need control to record myself and hear … that should be optional for the session (in case user cannot speak) but if it is on I want some automation here (auto play my version)”

The assistant proposed a session recording switch, Record → Stop & compare → automatic playback of the learner take → complete phrase/model playback → self-check, with persistent controls to replay either version and record another try. Recording-off practice uses Reveal & compare and manually requested model audio, with silent practice recorded distinctly from spoken performance. The user replied **“great”** and requested preserving all the solutions. See the [recording decision](decision-recording-and-comparison.md).

At that point, manual versus automatic stopping and retention remained unanswered; the general endorsement did not choose them. The resumed discussion below resolves both. Recording on/off defaults and final visual design remain open.

## Final consolidation request

The user specifically selected **`~/projects/phrase-gym/specs/`**, plural, as the home for all goals, functionality, decisions, research and plans. They asked that phrase-shadowing retain only the initial/legacy trainer content and that the next session need no context from this chat.

The consolidation therefore moves active development documents, preserves source history and stopped experiments in clearly labelled archives, adds new-session instructions, and leaves the application's scaffold unchanged. It does not restart implementation. The [migration record](migration-record.md) documents the move and verification.

## Resumed discussion and accepted stopping behaviour

Later on 2026-10-11, the user asked to continue discussing prototypes and functionality and to preserve all decisions in `specs/`. A simulated practice-screen walkthrough illustrated the agreed recording/comparison paths; it is a discussion sketch, not a passed P1 gate or approved visual design.

The user resolved the stopping choice:

> “this is in app settings (not implemented in 1st version), by default manual stop”

Manual **Stop & compare** is the default and the only stopping method in the first version. The choice of manual versus automatic stopping belongs in app settings in a later version; the selector and automatic stopping are deferred beyond the first version. This does not defer the session recording on/off switch or decide its default. The [recording decision](decision-recording-and-comparison.md) and active plans have been updated accordingly.

The user then clarified cue preparation:

> “we should have differently hidden words (if this is possible for the phrase, prepared by LLM when we add the phrase to the app DB), so on next session we could show different version.”

The LLM prepares alternate hidden-word/chunk cues during phrase intake. Check and store usable variants with the same phrase so later sessions can present different versions. Preserve enough identifying material and the whole-phrase response; do not force multiple variants when the phrase does not support them. One target and schedule cover every version. The [targeted-drilling decision](decision-targeted-drilling.md#prepare-variants-when-adding-the-phrase) and relevant preparation/selection plans now record this explicitly.

The user subsequently chose to keep **all recordings in the database**, with a future UI to compare progress, and explicitly deferred that UI. Store every take's audio and linked practice metadata automatically from the first version, including retries; no manual Save action, overwriting earlier takes or automatic expiry. This supersedes the assistant's selective-saving proposal. Immediate self/model comparison remains agreed core behaviour. The [recording decision](decision-recording-and-comparison.md#store-all-recordings-progress-comparison-ui-later), data contracts and implementation plans have been aligned.

## Unified session, explanation card and native language

The user rejected separate Learn new material and Rehearse home-screen actions. The main screen needs one **Start session** button. Phrases originate in source material and are expected to have been seen at least once; a newly added database item therefore does not require another automatic introduction. The drill must nevertheless assume the learner may not remember or understand the phrase and provide a visible button to open the complete phrase and an explanation in the learner's language. Normal drills intentionally avoid that language. This supersedes the previous mandatory new-item introduction and separate session-entry proposal; it does not remove prepared dialogues from scope.

The user asked to return to the drilling/explanation-card design. The current sketch proposes Show & explain, a focused full-phrase card with audio and Russian meaning/use, an expandable breakdown, and a return to the same drill with exposure retained. These labels and visual choices remain proposals.

The user selected **Russian for the sketch** and specified: ask for native language on app startup with an empty database and store it as an app setting. This setting supplies explanations on subsequent visits. It is separate from the language being learned, and Russian is not assumed for all users. The first-launch/settings layout remains undesigned.

## Compact drill and recording-button refinement

The user chose to keep the drill compact and explicitly removed “Say the whole phrase aloud.” and “Ready when you are.” Use an icon-only microphone button to start capture, with an accessible name; while recording, change it to a stop icon plus **Stop recording**. This replaces the earlier Stop & compare label without changing automatic learner-take/model playback after stopping. The complete phrase remains the response; only the repetitive on-card instructions are removed. The user then directed the discussion to the Show & explain card. Its detailed content and return behaviour are still proposals, not a blanket approval of the sketch.

The user then answered **yes** to recording a practice take directly on Show & explain with the complete phrase visible, followed by the same learner/model comparison. This accepts optional recording on that card over the read/listen-only alternative. Keep the phrase visible throughout, respect the session recording switch, retain every take in the database and mark it as practice with the model shown. It shares the target/history and cannot count as independent recall or erase prior help/difficulty. Exact placement and explanation content remain design proposals.

## Home controls, phrase lookup and lesson navigation

The user requested two controls on the starting screen: a lesson selector and phrase lookup. The lesson selector is a dropdown filtered by entered characters in the lesson title (`rap` finds Building rapport and other matching titles). Default to all lessons according to spaced repetition, keeping the single Start session action.

Phrase lookup is for remembering/finding an expression, not starting a review. It must support typed fragments and smart matching for likely concepts absent from the phrase's actual wording, such as `greetings`. Selecting a result opens Show & explain. That card must link to the phrase's lesson, opening a list of every phrase in the lesson; the list includes an action to start drilling that lesson.

These are accepted requirements, recorded in the [navigation/search decision](decision-session-navigation-and-search.md). Search architecture, result ranking and layout are proposals. An optional clarification asks whether selecting a lesson drills only due phrases or all its phrases; no answer has been recorded yet. The navigation sketch uses illustrative intent metadata and one example drill per lesson, not real semantic-search or scheduling validation. The source scaffold remains unchanged.

## Research-based answer card and scheduling request

The user asked for academic research on learning phrases/words, an answer card with appropriate self-reported results, an effective spaced-repetition algorithm, and other changes that help achieve ready use. The assistant recorded the [research](research-answer-card-and-scheduling.md) and [design recommendation](design/answer-card-and-review-policy.md). Recommended labels are Not yet / With effort / Recalled, separating correctness from extra practice and protecting ambiguous/assisted attempts. The recommended scheduling engine is FSRS with 0.90 default retention and ordinary automatic target-state updates. These exact choices have not yet been accepted by the user.

**Accepted correction:** the user rejected being asked to validate the method in their own learning: they want the best approach with scientific foundations, even where the evidence is imperfect. There must be no in-app validation option, experiments, special assessment sessions, A/B learning trials or calibration tasks. The assistant must choose sensible evidence-informed defaults; normal usage and normal answer ratings are enough. This withdraws the assistant's proposed personal effectiveness pilot and special 7/28-day checks. Engineering checks of software, devices and content remain the implementer's responsibility.
