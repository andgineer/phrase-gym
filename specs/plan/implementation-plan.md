# Phrase Gym — implementation roadmap

Updated 2026-10-11. **Written plan for a new session. No approved prototype or active app implementation exists; every phase gate remains unpassed.** The user stopped premature temporary implementation and requested design discussion first. Its archive, earlier sketches and legacy files do not satisfy any milestone. Read [START-HERE](../START-HERE.md).

The agreed learning-behaviour baseline is [targeted drilling](../decision-targeted-drilling.md), supplemented by [optional recording and automated comparison](../decision-recording-and-comparison.md). The app name is Phrase Gym; its home is `~/projects/phrase-gym`, with development docs under `specs/`. The [architecture](../architecture.md) and proposed defaults are recommendations to validate, not additional user-approved requirements.

## Delivery sequence

| Phase | User-visible result | Detailed work | Depends on |
|---|---|---|---|
| **P1 — Prototype and validate** | Review the discussed cue/response/help design, both recording modes and automated learner/model comparison on the phone. | [Prototyping](01-prototyping.md) | Agreed drilling decision only |
| **P2 — Foundation and content** | Open a real local app, import a reviewed collection and hear its model audio. | [Core implementation](02-core-implementation.md) | P1 findings and selected scope |
| **P3 — Complete core trainer** | Practise phrases/words/dialogues with optional recording, automated comparison, automatic database storage of every take, silent mode, pause/resume and durable due reviews. | [Core implementation](02-core-implementation.md) | P2 |
| **P4 — Phone reliability** | Reliably retain voice takes and complete a prepared session through a connection loss without losing recordings or duplicating reviews. | [Validation and release](04-validation-and-release.md) | P3; P1 audio probe |
| **P5 — Content preparation and intake** | Prepare reviewed material with AI and send selected words/phrases from echo-words into the same trainer. | [Content and integrations](03-content-and-integrations.md) | P2 contracts; P3 working drills |
| **P6 — Private release and reliability** | Use the app on a chosen private HTTPS host, with working audio, backups and a tested recovery path. No learner-run effectiveness pilot. | [Engineering checks and release](04-validation-and-release.md) | P3–P5; actual device evidence |

P4 and P5 can be developed independently after the core is stable. Choose the host in P6; a temporary development HTTPS endpoint needed for P1 is not a hosting decision.

## Recommended first release

Include fixed expressions and selected words/frames from the beginning, with reviewed partial cues, meaningful mask changes, introduction/model help, spoken self-check, repair, bounded dialogues and one target schedule. The first useful local slice is P3, including session recording/replay, automatic database storage of every take and silent mode; P4 adds device resilience, transfer recovery and offline completion. Historical progress-comparison UI is deferred. Recommend completing P5 before calling the source-replacement workflow finished.

Use a reviewed manual content package while building the core. This stages integration work without redefining the product as phrases-only. Support explicit language metadata throughout; use the learner's English material first, plus fixtures that catch assumptions about Serbian/German text. The actual launch language set remains a prototype/pilot choice.

Defer automatic speech grading, locked-screen/eyes-free operation, arbitrary offline session creation, old Anki history migration, social features and additional host infrastructure. These are not prerequisites for proving the agreed learning loop. Open-ended AI conversation remains excluded entirely.

## Working rules for implementation

- Each work package ends with its listed artifact and observable checks. Update status using evidence, not the presence of a mockup or an unchecked task list.
- Keep production and prototype code separate. Reuse a prototype component only after inspecting its contracts, behaviour, error paths and tests.
- Treat content, cue and scheduling changes as versioned changes. Preserve attempts and their actual support history.
- Test pure session/scheduling rules with controlled clocks and fixtures; test persistence through transaction/retry cases; use browser/device tests for actual user flows. Avoid tests that merely repeat rendered labels or mock every relevant behaviour.
- AI output needs real model evaluation on reviewed fixtures. Mocked provider calls test plumbing only. A model's self-check cannot certify cue quality.
- Future changes to echo-words follow its own repository instructions and checks at that time. This planning task changes no echo-words code or Anki data.
- No prototype, stack, host, migration or release is recorded as accepted merely because it appears in this plan. Ask for the user's judgment when there is a concrete prototype or deployment proposal to inspect.

## Milestone evidence to retain

Each phase records a short report in `specs/plan/evidence/` when it is actually performed: build/version, device/browser where relevant, tested scenarios, outcomes, remaining limitations, decisions and links to the runnable result. There are no completed reports yet. Keep personal recordings out of source control and these reports; use identifiers and consented examples.

## Next session and scaffold

Use the [next-session brief](next-session.md). The current task was discussion, documentation and consolidation only; the user intends to request implementation in a separate session. Continue unresolved design decisions with the user before treating those choices as approved. Do not resume or extract the stopped prototype as the baseline.

When implementation is requested, start **P1.1: prepare the small reviewed content and cue set**, then the interaction prototype and real audio probe. Preserve the user-created Python src layout, rich-click entry point, hatchling/uv packaging, pytest, Invoke, quality checks, documentation and CI workflows. Replace `src/phrase_gym/main.py` and other template examples only as part of authorized implementation. Do not create another `spec/` folder or replace the scaffold wholesale.

The interaction walkthrough must include the agreed optional recording and automated own-take → model comparison, not just a text-only demonstration. Use the accepted manual Stop recording in the first version; automatic stopping and its app-settings selector are deferred beyond that version. Validate the manual recording/comparison flow on the real phone and record remaining P1.4 decisions before production foundation work.
