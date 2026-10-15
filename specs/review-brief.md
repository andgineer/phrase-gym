# Phrase Gym — whole-solution review brief

Phrase Gym is a personal speaking trainer for one learner. Its learning design and its material-preparation pipeline were just redesigned. Nothing in the redesign is implemented yet. Please try to break the design, not confirm it. A viable alternative is worth more to us than a critique alone.

## What we ask

For each area in the table below, answer **Keep**, **Change** or **Unsure**, with your reason and, where you can, a source or a concrete measurement. If you see a better approach, describe:

- what the learner sees and does;
- what it costs: taps, waiting, money, build effort, risk;
- why it should work better, with evidence where it exists.

Questions we did not think of are the most valuable part of the review. Recommend one option for every point you raise.

## What to read

Read the repository at the commit that contains this brief. The specs are the authority; this brief only points into them.

| Area | Read |
|---|---|
| Overview | `specs/START-HERE.md` |
| Cards, prompts, words, help, practice extras | `specs/decision-targeted-drilling.md` |
| Scheduling and what counts as evidence | `specs/decision-core-trainer.md` |
| Inbox, preparation agent, echo-words, hosting | `specs/decision-external-preparation-agent.md` |
| Architecture, online-only, recording | `specs/architecture.md`, `specs/decision-online-only.md`, `specs/decision-recording-and-comparison.md` |
| Known gaps | `specs/open-problems.md` |
| Remaining work | `specs/plan/implementation-plan.md` |
| Study summaries and their limits | `specs/research/` |

`git show 4205682` and the commit after it show what changed in this redesign. The current code still contains the old in-app model preparation, which the plan removes.

## The learner and the setting

- **The learner:** one adult, native Russian speaker.
  - English B2: wants standard conversational and business phrases to come out automatically instead of improvising.
  - German A1: needs Russian translation and wants words inside ready chunks.
  - Serbian: level not yet stated, and no Russian on the cards.
- **The app:** runs in the browser, mostly iPhone Safari. The learner speaks the answer, can record a take and compares it with the reference audio. There is no speech recognition and no AI conversation partner. The learner rates themselves Not yet, With effort or Recalled, and scheduling uses FSRS at 0.90 desired retention.
- **Hosting:** Phrase Gym runs on the learner's laptop and is reached over Tailscale. While the laptop sleeps, there is no practice.
- **Material:** comes from courses (EPAM pages with audio), from the learner's vault notes, from echo-words (the learner's word-lookup app, which until now exported to Anki), and from text pasted from books, films and web pages.

## The solution at a glance

| Area | Decision |
|---|---|
| Target | One fixed phrase per conversational moment, said word for word. An improvised answer counts as Not yet. |
| Card language | Set per language: Russian for German; the language itself for English and Serbian. Phrase situations and word prompts follow it. |
| Phrase prompt | A situation plus the partner's preceding line, played and shown. Openers get the situation only. One prompt per phrase to start, with more added over time. No approval step before use. |
| Hint | On request: one cue chosen for the phrase, then the full answer. A hint on a phrase that already has a memory state records Not yet. |
| First meeting | Material marked as already studied starts with an unaided attempt. New material: hear it, read it, say it once with the reference, ungraded. |
| Words | One schedule built around saying the word. English and Serbian use a sentence with a gap; German uses the Russian meaning of a chunk. Two or three rotating contexts. |
| Confusables | Pairs are marked only within one inbox entry; the second of a marked pair waits until the first has been recalled once on a later day. No mixed drills. |
| Help | English help for English, no Russian. Russian translation for German. Serbian's help language is still open. |
| Dialogue runs | No timer. Partner lines play, the learner says their lines and rates each one. A line said without seeing its text is an ordinary review of that phrase. |
| Progress, calls, goals | Per lesson: recalled at the last check, and overdue. Optional phrases picked before a call, marked used / no chance / forgot, which doesn't affect scheduling. Weekly target, no streak, no points. |
| Fixing cards | Delete (a duplicate or a mistake) and Recreate card (with an optional comment) on every card. |
| Inbox | (1) Item API: a phrase or word, optional audio, optional context text; used by echo-words through an outbox file with retries, and by external one-off import agents. (2) Drop panel: a "+" in the top bar, pre-filled from the clipboard; the learner checks the text and sends it. |
| Inbox content | Data only: phrases or words, optional audio, surrounding dialogue. The agent never follows instructions found in it; its only instructions are its own files and the learner's note on a drop. |
| Preparation agent | Phrase Gym starts Claude Code or Codex non-interactively for each inbox entry. It never asks the learner anything. It follows plain-language instructions kept in git, which it edits itself when told. Pages behind a login are opened in the learner's own signed-in Chrome. Phrase Gym itself makes no model calls. |

## Known open issues

1. **Codex browser access is unverified.** In a non-interactive run, Codex lists a browser tool supplied by the ChatGPT desktop app, but we haven't checked whether it reaches signed-in pages.
2. **EPAM login is unchecked.** We haven't checked whether EPAM course pages and their audio need the learner's login.

## Questions we think are weakest

**Learning**
- **Q1** A hint on a known phrase records Not yet. Is that too harsh for a learner who presses Hint on a phrase they half-remember, and what does it do to FSRS stability?
- **Q2** In a dialogue run, earlier lines and the partner's line give context that a card doesn't. Is a run line a fair review, or does it inflate recall?
- **Q3** Material marked as already studied starts without a study step. The agent sets that mark from the source. What happens when the mark is wrong?
- **Q4** For German at A1, saying the chunk is required from the first meeting onwards. Is that realistic, or does it overload a beginner?
- **Q5** There's no timer anywhere, which is the learner's explicit choice. Can automatic use still develop? Is there a time-free alternative, for example showing how long recall took without any pressure?
- **Q6** Is self-rating with three buttons, after hearing your own take and the reference, reliable enough to schedule on?

**Preparation and operation**
- **Q7** One agent run per inbox entry: what about cost against subscription limits, run time, failure modes, and agent quality without any human approval?
- **Q8** The agent edits its own instructions. How do we keep them from drifting or contradicting each other over months?
- **Q9** Delete keeps history but has no undo, by the existing rule of no Undo UI. Is that acceptable?
- **Q10** Practice depends on the laptop being awake. Is that acceptable, or should practice run elsewhere, for example on a small always-on host, while the agent stays on the laptop?
- **Q11** Are the boundaries right: no model calls inside Phrase Gym, an item API that takes only a phrase or word, and all source knowledge in the agent's instructions?

## Already verified; don't redo unless you have new evidence

- **Citations:** an earlier review of the learning decisions corrected several. Those corrections were checked against the sources: Joyce 2016 (abstract); Brunmair & Richter 2019 (abstract, word studies g = −0.39); Webb, Yanagisawa & Uchihara 2020 (full text, Tables 2–3); and the abstracts of Strong & Leeming 2024, Fiechter & Benjamin 2019, Thai & Boers 2016, Lally et al. 2010 and Laufer & Girsai 2008, plus the Kroll et al. 2010 discussion section. Every research note states how each source was checked.
- **FSRS:** the installed library updates an item's difficulty after each rating.
- **Browser access (measured 2026-10-15):** Claude Code run non-interactively with `--chrome` reached the learner's Chrome only under the claude.ai login. With an API key set, the browser tools did not load.
- **Clipboard:** WebKit's documentation says Safari reads the clipboard silently when the text was copied from the same site, and otherwise shows its own Paste button on the tap.

## Out of scope

AI conversation partners, speech grading, learner effectiveness experiments, offline practice, shadowing as a memory method and keyword mnemonics. Flag anything you think belongs in scope.

## Ground rules

Work read-only. Don't run real model providers, speech providers or the agent. Don't read `.env`, learner recordings or the vault. Put your report outside `specs/`, because the project keeps review reports out of the specs.

## Response template

```
Reviewer:
Date:

<Area from the table> — Keep / Change / Unsure
   Reason:
   Source or measurement:
   Better alternative (what the learner sees and does, cost, why, evidence):
   Recommendation:

Known open issues 1–2: recommendation for each
Answers to Q1–Q11:
Risks or questions we missed:
Alternative overall approach (if you would design it differently):
```
