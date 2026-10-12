# Phrase Gym — start here

**Latest accepted interaction, 2026-10-12:** follow the [unified practice-card decision](decision-unified-practice-card.md). One card retains recording, replay, reference and colored/icon self-check controls; answer reveal replaces gaps, explanation expands inline beside Show answer. Remove the recording-mode switch, Practice again and supported-only Continue. The microphone/stop icon reflects the active audio action. This supersedes conflicting earlier flow details below.

Handoff updated **2026-10-12**. The complete project context is in `~/projects/phrase-gym/specs/`. Read this file before using any earlier plan or archive.

**Current work, 2026-10-12:** the user explicitly requested implementation of **P1**. New disposable fixtures, interaction prototype and audio/storage probe now live in `prototypes/`. See [run/walkthrough instructions](../prototypes/README.md), [P1 evidence](plan/evidence/p1-2026-10-12.md) and [review handoff](plan/p1-review.md). Software checks pass; user walkthrough, actual iPhone/device checks and authentic actor/Serbian audio remain pending. **P1 is not passed and P2 has not begun.** The Python application scaffold is preserved. Earlier discussion-only restrictions describe the prior session, not this new authorization.

**Latest manual feedback:** UI-language switching is required in version one, recording is optional directly on the unified card without a mode switch, full-answer help is labelled **Show answer**, **Other answer / problem** and **I couldn’t recall it** are removed, and **Show & explain** is a visually distinct action beside Show answer that expands materials inside the card. See [accepted feedback](decision-p1-manual-feedback.md).

**Latest review follow-up:** three fresh Sol 6.1 reviews found and corrected P1 defects; see [the review-loop report](plan/evidence/p1-review-loop-2026-10-12.md). Pause before a fourth automated review; the recommended next P1 step is the user’s manual walkthrough. The user rejects macOS-only audio dependence, asks to reuse echo-words’ Piper/Edge engines and llmbroker, and authorizes commits directly to `main`. Production source remains a P2 scope decision.

**Prior documentation handoff, 2026-10-11:** the user asked to put everything discussed into the implementation plan. The [consolidated implementation plan](plan/implementation-plan.md) now collects the full product flow, language/bookmark requirements, recording retention, research-based defaults, echo-words audio guidance, architecture, phase coverage, acceptance checklist and remaining choices. The [next-session brief](plan/next-session.md) is the short entry to that plan. This request did not authorize application implementation or blanket approval of every recommendation.

**Prior design session, 2026-10-11:** the user asked to continue discussing app prototypes and functionality and to preserve all decisions in `specs/`. Design discussion has resumed; application implementation has not been requested. See the [current discussion record](design/interaction-design.md#resumed-prototype-discussion-2026-10-11). Continue recording accepted choices in the relevant decision documents and distinguish proposals from acceptance.

**Latest research/design request:** the user asked for academic evidence on learning words/phrases, an answer card and an effective spaced-repetition design. See the [evidence review](research-answer-card-and-scheduling.md) and [recommended answer card/review policy](design/answer-card-and-review-policy.md). **Accepted correction:** choose scientifically grounded usable defaults; do not ask the learner to validate effectiveness or add in-app experiments, special assessment sessions or calibration tasks. Exact three-button labels and FSRS settings remain recommendations, not user-approved choices.

**Research must affect the design:** the user then challenged whether the findings had actually changed the app. The [concrete before/after changes](design/answer-card-and-review-policy.md#what-the-research-changes-in-the-design) now flow through the interaction design, learning engine, data contracts and implementation handoff: explicit recall grading, adaptive intervals, separated bounded retries, purposeful chunk coverage, varied prepared situations, meaning repair and honest silent/helped-attempt handling. Do not fall back to old Got it/Try again labels, the fixed ladder or cosmetic cue rotation as the current recommendation. Earlier visual sketches remain historical where they differ.

**Latest language/audio requirements:** the user rejected **Model** as an audio-button label; **Reference** is the recommended replacement for actor/course recordings and generated audio alike. Native language supplies explanations. UI language can follow either native language or selected learning language, without a separate arbitrary-language selector. Add a learning-language selector and bookmarkable URLs for concurrent study of several languages. See the [language and audio decision](decision-languages-and-audio-labels.md) for accepted requirements and proposed routes/preferences.

**Audio implementation reference:** at the user's request, the nearby echo-words source was inspected for multilingual speech. [Findings and adaptation guidance](reference/echo-words-audio.md) cover its English/German Piper voices, Serbian Edge voice with Cyrillic speech input, human-audio priority, cache and fallback rules, and differences needed for complete phrase recordings. Preserve supplied actor audio; the inspection is not application implementation or a new quality benchmark.

## What the user wants

**Phrase Gym** trains selected words in useful phrases, fixed expressions and specific prepared dialogues until they can be used readily in the intended situations. The purpose is automatic access to useful language. It is not a general AI speaking partner: open-ended conversation and free role-play were explicitly rejected, including as future directions.

The user wanted design discussion in the current conversation, followed by a written plan for implementation in **a new session with a different model**. They explicitly stopped an assistant that began coding before discussing the design. Do not infer permission to resume that work from its existence or from older messages asking to “start.” The present handoff is documentation and project organization only. Follow the new session's actual user request for the next action.

Key user statements:

> “the app is to drill words in phrases or specific dialogs with specific phrases I should learn to the automatic usage in specific situations.”

> “it should for each specific phrase find what part is enough to show so it could not be confused with any other variants”

> “need control to record myself and hear … optional for the session … if it is on I want some automation here (auto play my version)”

> “after that you will write all our decisions to the plan for new session”

## Read in this order

1. [Accepted targeted-drilling decision](decision-targeted-drilling.md): scope, introductions, phrase-specific cues, varying hidden chunks, whole-phrase responses, alternatives, support and one schedule per target.
2. [Recording and comparison decision](decision-recording-and-comparison.md): the latest endorsed interaction flow, optional session recording, automated self/model playback and manual stop by default; the stop-method setting is deferred beyond the first version.
3. [Session navigation and search](decision-session-navigation-and-search.md), [languages and audio labels](decision-languages-and-audio-labels.md), then [interaction design](design/interaction-design.md): language/lesson selection, smart phrase lookup, lesson links and concrete screens, with agreed behaviour separated from visual proposals.
4. [Implementation roadmap](plan/implementation-plan.md): current status, scaffold constraints and future phases. [Next-session brief](plan/next-session.md) is the implementation handoff.
5. [Architecture](architecture.md), [learning engine](architecture/learning-engine.md), [data contracts](architecture/data-contracts.md): technical recommendations, not automatically approved decisions.
6. [Functional description](functional-description.md), [open choices](plan/functionality-discussion.md), and [research](research-memorizing-phrases.md): supporting scope and evidence limits.

## Current project state

The user created a Python scaffold under `~/projects/phrase-gym`. It contains `src/phrase_gym`, a rich-click CLI entry point, hatchling/uv packaging, pytest, Invoke, pre-commit/type-check tooling, documentation and CI/release workflows. The `main.py` file-conversion command and other cookiecutter examples are **templates to replace later**, not app behaviour. The scaffold's source, tests, dependency files and workflows were left unchanged by this consolidation.

The **new P1 prototypes are runnable but not yet user-approved**; production app source remains the scaffold. All roadmap phase gates remain unpassed. See the current evidence above. The old `phrase-shadowing/index.html` and `phrases.py` are legacy source tooling, not the new app and not progress to migrate.

The stopped attempt produced temporary source, fixtures and a browser prototype outside both project folders. It was never installed into this scaffold and its server was stopped. A labelled archive preserves that work so the context does not depend on `/private/tmp`; it is not a starting baseline, approved design or completed P1 milestone. The earlier visual sketches have the same non-authoritative status. See [archive inventory](archive/README.md).

## Decisions that must survive the handoff

- Name: **Phrase Gym**. Repository description: “A language trainer for automatic use of words and phrases through guided recall, spaced repetition, and targeted dialogue drills.”
- Train both words in useful phrases and fixed phrases/constructions; rehearse specific dialogues with defined targets. Source rollout remains a separate choice.
- The main screen starts one session; no separate Learn new material or Rehearse entry modes. Phrases are expected to have been encountered in source material. Start from the drill, with a visible action to show the complete phrase and an explanation in the learner's language when forgotten or unclear. No compulsory app introduction for every newly added item.
- Home has a searchable lesson dropdown, defaulting to All lessons under spaced repetition; title substrings such as `rap` find Building rapport. A separate phrase lookup also matches concepts absent from the wording, such as `greetings`, and opens Show & explain without starting a review. That card links to its lesson's complete phrase list, which has a Start lesson drill action. Selected-lesson due-only versus all-phrase policy remains open; see the navigation decision.
- Normal drills use the target language; the learner's language belongs on the explicitly opened explanation card. Broad situations cannot demand an arbitrary exact sentence. Keep track of explanation/model exposure when returning to practice.
- First launch with an empty database asks for native language and stores it in app settings for explanations. Russian is selected for the current sketch; it is not a hard-coded app default.
- The learner can study several languages concurrently. A learning-language selector sets a bookmarkable URL; sessions, lessons and search use that language. Native language remains the explanation language. From version one, UI controls may use native language or follow the selected learning language; do not add an unrelated UI-language dropdown. Target-language drill content is separate from UI chrome.
- Do not label comparison audio **Model**. **Reference** is the current recommended wording; the recording can come from a professional actor/course or speech synthesis. Preserve its source and the existing learner-first comparison order.
- Use enough meaningful, phrase-specific cue material from any part of the expression. Show necessary disambiguation initially. Deliberately vary hidden chunks when useful; no random masks or letter puzzles.
- When adding a phrase to the app database, have the LLM prepare alternate hidden-word/chunk cues where useful variants are possible. Check and store them with the phrase so a later session can show a different version. Do not force multiple variants or generate random gaps during practice.
- The response is the whole phrase. Fixed targets have explicit equivalents; word/frame tasks define what may vary.
- Ambiguous cues and valid alternatives are not memory failures. Self-comparison remains available; AI/ASR grading is not required.
- Preserve support/exposure history. Reading a model, immediate echo, partial-cue retrieval, shown-word usage and silent practice are different evidence.
- One learning target has one schedule across examples, cue variants and dialogue appearances.
- Provide recording/replay controls directly on the practice card, without a session-level recording switch; self-check also works without recording. The endorsed working order is learner recording → complete phrase/model → self-check.
- Show & explain also supports recording with the complete phrase visible throughout capture and learner/model comparison. These are model-supported practice attempts, stored like all other takes and linked to the same target/history. Recording is directly available without a mode switch.
- Manual stopping is the default and the only stopping method in the first version. The compact drill starts capture with a microphone-icon button; during capture it becomes a stop icon plus **Stop recording**. Omit “Say the whole phrase aloud.” and “Ready when you are.” The complete-phrase task and automatic comparison remain unchanged. Choosing manual versus automatic stop belongs in later app settings; the selector and automatic stopping are deferred beyond the first version.
- Automatically store all learner recordings, including retries, as audio and metadata in the app database from the first version. No manual Save action or automatic expiry. A UI for comparing progress across historical recordings is deferred; immediate self/model comparison remains core.
- Recording off permits practice when the learner cannot speak. It must not require microphone permission or imply demonstrated spoken performance; audio is manually requested in this mode.
- No deployment, Git initialization, course enrollment, echo-words output switch or Anki migration is authorized by this handoff.

## Still open

Final visual styling, remaining navigation details, selected-lesson review policy, search implementation, platform/device feasibility, review policy numbers, first-source rollout and hosting remain open. Manual recording stop and automatic retention of all recordings in the database are accepted. The stop-method setting, automatic-stop detection details and historical recording-comparison UI are deferred. The earlier endorsement of the recording flow does not turn every previous mockup or technical assumption into a decision.

Keep the context focused: consult active documents first. The archives preserve historical material, including rejected ideas and misleading claims from the aborted implementation attempt, so they must never override this handoff.
