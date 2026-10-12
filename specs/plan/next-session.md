# Brief for the next session

Updated **2026-10-12**. The user has now requested P1 implementation; new disposable deliverables are available. Start with [P1 evidence](evidence/p1-2026-10-12.md), [the review handoff](p1-review.md) and [prototype walkthroughs](../../prototypes/README.md). User/device/content acceptance remains pending; P2 has not begun. The following handoff retains the original requirements and execution sequence.

## Read first

1. [START-HERE](../START-HERE.md) and [agent guidance](../agent-guide.md).
2. **[Consolidated implementation plan](implementation-plan.md)** — the complete scope, current defaults, phase sequence, acceptance checklist and remaining choices. This is the main execution handoff; do not reconstruct requirements from chat history or earlier sketches.
3. The relevant linked phase file when beginning that phase: [P1 prototype](01-prototyping.md), [P2–P3 core](02-core-implementation.md), [P5 content/integrations](03-content-and-integrations.md), [P4/P6 reliability/release](04-validation-and-release.md).

The user required design discussion and a written plan before implementation in a new session. The latest requests authorized P1 implementation and reviews, followed by direct-main commits and a preference for manual prototype review. **P1 prototypes have been implemented; none is yet user-approved and no phase gate has passed.** Follow the new session's actual request; do not automatically start coding because this handoff exists.

## Requirements that must survive

- Train selected words in useful phrases, fixed expressions and specific prepared dialogues. Always request the whole phrase with enough meaningful identifying material. No free conversation, role-play, random masks or letter puzzles.
- One Start session. Prior source exposure is expected, but Show & explain is available immediately when material is forgotten or unclear. No mandatory introduction for every new DB item and no Learn/Rehearse split.
- Ask for native language at first launch and store it for explanations. A learning-language selector has bookmarkable URLs. UI can use native language or follow the learning language, independently of explanations; no arbitrary UI-language dropdown. Scope lessons/search/sessions to the selected learning language and preserve each language's progress.
- Home has searchable lesson selection, default All lessons under spaced repetition, and separate concept-aware phrase lookup. `rap` finds Building rapport; `greetings` can find phrases without that literal word. Lookup opens Show & explain, which links to the lesson's full phrase list and Start lesson drill. Browsing is not a graded review.
- Keep the drill compact: mic icon only to start; stop icon plus Stop recording during capture; no “Say the whole phrase aloud.” or “Ready when you are.” Manual stop is the only V1 method; its automatic-stop setting is deferred.
- Recording is optional per session. When on: learner take → automatic own playback with cue retained → full phrase/reference playback → self-check. Recording off requires no microphone and keeps reference audio manual. **Model is rejected as an audio-button label; Reference is the working replacement.**
- Show & explain also permits recording, with the full phrase visible throughout capture and comparison. Preserve support history and the original attempt's result.
- Keep **all learner takes' audio and metadata in the DB from V1**, including rerecordings and recoverable interrupted clips. No manual Save, overwrite or automatic expiry. Historical progress-comparison UI is deferred; immediate playback remains core.
- The LLM prepares checked alternate hidden-word/chunk cues at phrase intake where useful; store for later sessions. One reliable cue is acceptable. All cues/examples/dialogue appearances share one target schedule.
- Preserve course/actor audio; generate missing whole-phrase reference clips during preparation, never as a dependency of advancing a drill. Follow the inspected [echo-words audio guidance](../reference/echo-words-audio.md), including Serbian speech-only Cyrillic conversion and matching-language fallback.
- The learner will not run effectiveness experiments, special assessment sessions, calibration tasks or tune a learning algorithm. Choose usable scientific defaults; engineering/content/device checks are the implementer's work.

## Concrete recommendations to carry forward

The master plan includes the proposed Not yet / With effort / Recalled outcomes, separate extra practice, FSRS at 0.90 desired retention, bounded separated retries, purposeful chunk coverage and varied prepared contexts. It also proposes `/learn/{language}`, a native/learning UI-mode toggle, a due-only selected-lesson default with optional Practice all, and per-language reference-voice configuration.

Keep these recommendations distinct from explicitly agreed requirements. Use them to build a concrete prototype rather than ask the learner to invent parameters. The selected-lesson policy, recording on/off default, final styling, exact preference presentation, session budget and release/host choices remain listed in [remaining choices](implementation-plan.md#remaining-choices-and-proposed-defaults). Do not treat request for consolidation as blanket acceptance of all details.

## Start when implementation is requested

P1.1/P1.2 deliverables and the P1.3 probe now exist. Continue with their walkthrough/device/content review rather than rebuilding them. The original sequence was a small reviewed content/cue pack, then the complete interaction prototype and actual-device audio/storage probe. Include language/bookmark navigation, search/lesson routes, revised answer/help states and both recording modes, not just a text-only card. Earlier HTML illustrations simulate audio, contain some obsolete labels and do not implement the newest language behaviour; their existence grants no phase credit or visual approval.

P2–P3 build the local trainer using reviewed manual packages. P3 must already persist every recording; that is not postponed to reliability work. P5 supplies the agreed add-time LLM preparation and live source handoff before claiming the complete planned feature set. Record phase evidence and user walkthrough corrections under `specs/`; do not represent software tests as a study of the learner.

## Preserve the workspace and external sources

The real starting point is the user's scaffold: `src/phrase_gym`, rich-click entry point, hatchling/uv, pytest, Invoke, quality tooling, documentation and CI/release conventions. The current CLI file-conversion behaviour is a template to replace during authorized implementation. Proposed production stack is Vue/TypeScript, FastAPI and SQLite with learner audio BLOBs and local reference media; it is not installed app functionality.

There was no Git repository at handoff; do not initialize Git merely to run checks. All development context stays under `specs/`, plural. Leave legacy phrase-shadowing, vault notes, echo-words and Anki data unchanged unless their modification is actually requested. Deployments, source-output switches and course enrollment are not authorized by a planning document.

The stopped temporary implementation and its test/build results are [archived history](../archive/README.md), not a baseline to resume. Historical lockfile/documentation-build observations do not justify unsolicited dependency upgrades; reproduce any relevant issue during actual implementation. The local research, decision, architecture and audio-reference documents preserve the needed context without another chat-history hunt.
