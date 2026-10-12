# Phrase Gym — functional description

**Accepted P1 feedback, 2026-10-12:** UI-language switching is required in version one; recording defaults on with explicit microphone start; full-answer help is labelled Show answer; Other answer / problem is removed entirely. See [the decision](decision-p1-manual-feedback.md).

Updated 2026-10-11. **Draft supporting scope. The agreed baseline is [targeted drilling](decision-targeted-drilling.md); there is no approved prototype or active app implementation. [Optional recording and comparison](decision-recording-and-comparison.md) is an additional agreed interaction requirement.** Other features below remain recommendations for design discussion and [prototyping](plan/01-prototyping.md), not additional approved requirements. See the [architecture](architecture.md) and [implementation roadmap](plan/implementation-plan.md) for the proposed work.

## Purpose and learning targets

A personal trainer for automatic, appropriate use of selected expressions in specific situations, across the languages the user learns. It is intended to provide its own drilling and review workflow, with a future intake from echo-words.

An item is a fixed phrase, or a selected word sense or reusable construction learned through useful phrases. Words are not practised in isolation. Each item has:

- the target expression or construction, its language and intended use;
- a sentence or short dialogue showing that use, with reference audio and source (imported actor/course recording or generated speech);
- the wording that must be recalled and any permitted substitutions or equivalents;
- phrase-specific cues, including versions that hide different meaningful chunks;
- one review history covering all its exercises and dialogue appearances;
- an explanation in the learner's language available on request, an optional personal note and the learner's recordings.

## Sources

- Phrase banks and dialogues in the Obsidian vault, with existing actor or synthetic audio.
- echo-words selections, including the selected sense and useful examples, sent to the trainer instead of Anki.

Both source types are in the product's scope. Their first-version rollout and any migration of old Anki material remain undecided.

## Core practice

The learner is expected to have encountered the phrase in its source material at least once. A session can therefore begin with a recall attempt: its purpose or partner line and enough phrase-specific material identify the intended expression. The learner says the **whole phrase**, compares with the model and repairs difficulties. If the phrase is forgotten or not understood, a button on the drill opens the complete phrase and an explanation in the learner's language. No mandatory in-app introduction is needed simply because an item is new to the database. Later attempts can hide different chunks of the same target.

Cues are chosen separately for each expression. They need not start at the beginning. Additional help can reveal more meaningful material or the full model. Required disambiguating cues are present from the start; explanations and translations remain on request.

When a phrase is added to the app database, the LLM prepares cue variants hiding different words or meaningful chunks wherever useful alternatives are possible. Checked variants are saved with the same phrase for reuse, allowing the next practice session to show a different version. Every version still requests the whole phrase. Reuse a single reliable cue when the phrase does not support alternatives; do not force ambiguous masks. All versions share one target and schedule.

Research-informed refinement: choose among suitable stored cues to exercise under-practised or previously difficult chunks, preserving clarity and comparable support. For word targets, hide the actual word form during a word-retrieval attempt. Include suitable different prepared situations/dialogue turns in normal reviews, retaining the same target meaning and one schedule. This makes variation serve productive use; it does not introduce free role-play or compulsory zero-hint tasks.

Specific dialogues are learned and rehearsed by side. Partner lines play; selected target lines are recalled with suitable cues; supporting lines can remain available to read. A difficult exchange can be practised separately before returning to the dialogue. Controlled changes of details can exercise a word or frame without changing the learning objective.

The [decision record](decision-targeted-drilling.md) defines cue quality, answer handling, the distinction between assisted and independent production, and AI's bounded role. The [research record](research-memorizing-phrases.md) states the evidence and its limits.

## Review and progress

One schedule belongs to the learning target, not to each cue or exercise. The [current research-based recommendation](design/answer-card-and-review-policy.md) specifies the following design, pending user acceptance of the exact controls/policy:

- Rate the original attempt **Not yet / With effort / Recalled**; the latter two both mean correct recall. Keep Practice again separate from the grade. Extra pronunciation practice does not imply forgotten wording, and a successful retry does not overwrite the first result.
- Use **FSRS at 0.90 desired retention**, with automatic target-state updates from normal eligible ratings and actual elapsed time. This replaces the fixed interval ladder, universal one-day lapse reset and permanent 30-day ceiling. The learner does not tune or validate the method.
- After a miss, give corrective feedback and optional supported practice, then a retrieval retry after intervening material and time: roughly 5–10 minutes and at least two other targets when feasible, otherwise a later session. Aim for one successful separated retry, cap at two extra retrieval attempts, and revisit meaning/cue quality after persistent difficulty.
- Preserve the original review result while logging every retry and exposure. Deliberate silent recall may inform memory scheduling; model-visible repetition, reading and immediate imitation cannot become independent-recall successes. Ambiguous cues, technical failures and skips receive no failure grade.
- Keep cumulative chunk coverage to guide future cue selection. Do not force complete coverage again at every interval before the schedule can advance.

Progress must distinguish visible-model repetition, partial-cue recall and independent recall or use. Accuracy and hesitation may inform practice, but faster delivery alone does not establish mastery. No fixed sequence of exercise levels or universal countdown is required.

## Supporting capabilities

Automatic retention of all recordings is agreed for the first version. Other supporting capabilities below remain candidates unless separately accepted:

- **Recording history — storage now, comparison UI later:** automatically keep every take's audio and metadata in the app database, including retries, linked to its target, date, cue and practice conditions. No manual Save action or automatic expiry. A UI for comparing historical recordings/progress is deferred; immediate self/model replay remains core.
- **Explanation card — agreed:** a visible action on the drill shows the full phrase and an explanation in the learner's language. The learner can also record a practice take here with the full phrase visible, followed by the same automatic learner/model comparison. The session recording switch still applies. Store these takes and classify them as practice with the model shown. Normal drills keep learner-language translations/explanations hidden; returning retains the help history. Exact layout, depth and additional word-level help remain design choices.
- **Help with a difficult item:** useful examples, a personal mnemonic, or a reliable explanation of word origins, related words and common confusions. AI may assist with preparation.
- **Speech feedback:** optionally identify dropped or changed target words. Handle uncertain recognition explicitly and retain self-checking against the model.
- **Library:** basic smart phrase lookup and navigation to the lesson's complete phrase list are now agreed. Broader library views for sources, notes and progress remain candidates; historical recording-comparison UI remains deferred.

## Session formats and languages

The app supports one learner studying multiple foreign languages concurrently. Provide a learning-language selector and encode the selected language in the URL for bookmarks such as Phrase Gym: English and Phrase Gym: Serbian. The [language/audio decision](decision-languages-and-audio-labels.md) proposes `/learn/en` and `/learn/sr`. Scope the lesson list, phrase search and review session to this language; keep progress for every language in the same database.

The main screen has one **Start session** action, without separate learning-new-material and rehearsal modes. A searchable lesson dropdown defaults to **All lessons**, drawing phrases according to spaced repetition; entered characters filter lesson titles (`rap` → Building rapport). Selecting a lesson limits the session to that lesson; whether this includes due phrases only or every phrase remains open. Exact queue mixing and duration also remain open.

A separate **Find a phrase** control supports both text fragments and smart matching for concepts absent from the wording, such as `greetings`. Opening a result goes directly to Show & explain for reference; browsing alone does not complete a review or advance its schedule. That card links to its lesson's list of all phrases, which includes a Start lesson drill action. See the [navigation/search decision](decision-session-navigation-and-search.md) for accepted requirements and implementation proposals. Learning/explanation remains available from every drill. Short daily sessions and eyes-free practice are desired; hands-free controls remain open.

Normal drill phrases, situations and partner lines stay in the selected learning language. The explanation card uses the native language for meaning, translation and usage. Interface controls are separate: they may use the native language or follow the current learning language, with no independent arbitrary-language dropdown. A binary settings preference is the recommended presentation. The product is intended for several target languages, including English, Serbian and German, with appropriate reference recordings; the content/translation inventory remains a release detail. Do not infer explanation language from learning language or UI language.

On first launch with an empty database, ask for the learner's native language and save it as an app setting. Use that saved language for phrase explanations on subsequent visits. The native language and each phrase's target language are separate. The current sketch uses Russian as requested by the user.

Reference-audio controls use the working label **Reference**, paired with **My recording**. The user rejected **Model** because it suggests generated speech. Source/performer metadata is independent of button wording, and existing course/actor audio is preserved.

## Boundaries

Open-ended AI conversation, free role-play and a general AI speaking partner are excluded. Grammar courses, long reading exercises, social sharing and exporting to Anki are also outside this concept.

The next implementation phase is [prototyping](plan/01-prototyping.md). Architecture and implementation plans are recommendations; resolve remaining functional choices through concrete design discussion and engineering checks. The user rejects learner-run effectiveness validation, experiments and special assessment sessions. Choose scientifically grounded usable defaults and adapt scheduling from ordinary ratings. Deployment remains a later phase.

## Agreed recording and silent-practice modes

Provide an optional session-level recording switch. With recording on, the learner records the complete response, then the app automatically plays the learner's take, reveals the complete phrase and plays the model before self-check. Keep controls for replaying either version and recording another try. With recording off, support silent practice through Reveal & compare; model audio plays only on request. Record the actual practice modality and do not infer spoken performance from silent work.

Manual Stop recording is the accepted default and the only stopping method in the first version. Choosing manual versus automatic stop after a pause belongs in app settings in a later version; the selector and automatic stopping are deferred beyond the first version. Recording on/off remains a separate session-level switch. See the [decision](decision-recording-and-comparison.md) and [concrete screen flow](design/interaction-design.md).
