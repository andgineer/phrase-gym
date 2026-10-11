# Phrase Gym — functional description

Updated 2026-10-11. **Draft supporting scope. The agreed baseline is [targeted drilling](decision-targeted-drilling.md); there is no approved prototype or active app implementation. [Optional recording and comparison](decision-recording-and-comparison.md) is an additional agreed interaction requirement.** Other features below are candidates to validate through [prototyping](plan/01-prototyping.md), not additional approved requirements. See the [architecture](architecture.md) and [implementation roadmap](plan/implementation-plan.md) for the proposed work.

## Purpose and learning targets

A personal trainer for automatic, appropriate use of selected expressions in specific situations, across the languages the user learns. It is intended to provide its own drilling and review workflow, with a future intake from echo-words.

An item is a fixed phrase, or a selected word sense or reusable construction learned through useful phrases. Words are not practised in isolation. Each item has:

- the target expression or construction, its language and intended use;
- a sentence or short dialogue showing that use, with model audio and source;
- the wording that must be recalled and any permitted substitutions or equivalents;
- phrase-specific cues, including versions that hide different meaningful chunks;
- one review history covering all its exercises and dialogue appearances;
- optional explanations, a personal note and the learner's recordings.

## Sources

- Phrase banks and dialogues in the Obsidian vault, with existing actor or synthetic audio.
- echo-words selections, including the selected sense and useful examples, sent to the trainer instead of Anki.

Both source types are in the product's scope. Their first-version rollout and any migration of old Anki material remain undecided.

## Core practice

The learner first hears and sees the complete expression in context. A recall attempt then provides its purpose or partner line and enough phrase-specific material to identify the intended expression. The learner says the **whole phrase**, compares with the model and repairs difficulties. Later attempts can hide different chunks of the same target.

Cues are chosen separately for each expression. They need not start at the beginning. Additional help can reveal more meaningful material or the full model. Required disambiguating cues are present from the start; explanations and translations remain on request.

Specific dialogues are learned and rehearsed by side. Partner lines play; selected target lines are recalled with suitable cues; supporting lines can remain available to read. A difficult exchange can be practised separately before returning to the dialogue. Controlled changes of details can exercise a word or frame without changing the learning objective.

The [decision record](decision-targeted-drilling.md) defines cue quality, answer handling, the distinction between assisted and independent production, and AI's bounded role. The [research record](research-memorizing-phrases.md) states the evidence and its limits.

## Review and progress

One schedule belongs to the learning target, not to each cue or exercise. Reviews recur after delays; failures and support use inform subsequent practice. The exact algorithm, intervals, session limits and progress criteria are still to be specified.

Progress must distinguish visible-model repetition, partial-cue recall and independent recall or use. Accuracy and hesitation may inform practice, but faster delivery alone does not establish mastery. No fixed sequence of exercise levels or universal countdown is required.

## Supporting capabilities

These are desired capabilities; their first-version inclusion remains open:

- **Saved recordings:** keep selected takes or an early baseline for later comparison; retention and saving policy remain open. Session recording/replay itself is agreed core behaviour, described below.
- **Contextual help:** request a simple target-language explanation or a translation for a word or sentence. Translations are absent by default.
- **Help with a difficult item:** useful examples, a personal mnemonic, or a reliable explanation of word origins, related words and common confusions. AI may assist with preparation.
- **Speech feedback:** optionally identify dropped or changed target words. Handle uncertain recognition explicitly and retain self-checking against the model.
- **Library:** find an item and its sources, model audio, notes, recordings and progress.

## Session formats and languages

The app should support learning new items, reviewing due targets and rehearsing specific dialogues. Short daily sessions and eyes-free practice are desired; session duration and hands-free cue delivery, help requests and outcome reporting remain to be designed.

Learning material stays in the target language. The product is intended for several languages, including English, Serbian and German, with appropriate voices and on-demand translation. Which languages ship first remains part of the first-version decision.

## Boundaries

Open-ended AI conversation, free role-play and a general AI speaking partner are excluded. Grammar courses, long reading exercises, social sharing and exporting to Anki are also outside this concept.

The next implementation phase is [prototyping](plan/01-prototyping.md). Architecture and implementation plans are recommendations; remaining functional choices are resolved through concrete prototypes and pilot evidence. Deployment remains a later phase.

## Agreed recording and silent-practice modes

Provide an optional session-level recording switch. With recording on, the learner records the complete response, then the app automatically plays the learner's take, reveals the complete phrase and plays the model before self-check. Keep controls for replaying either version and recording another try. With recording off, support silent practice through Reveal & compare; model audio plays only on request. Record the actual practice modality and do not infer spoken performance from silent work.

The stop-by-button versus stop-on-silence choice is unresolved; manual Stop & compare was recommended. See the [decision](decision-recording-and-comparison.md) and [concrete screen flow](design/interaction-design.md).
