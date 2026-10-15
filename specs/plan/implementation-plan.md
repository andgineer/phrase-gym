# Remaining implementation work

This plan contains unfinished work only. Remove entries as they are completed; lasting constraints belong in the linked decisions, and code/tests describe delivered behavior.

## Preparation inbox and agent

Follow the [preparation decision](../decision-external-preparation-agent.md).

- **Remove in-app text preparation:** vault/legacy-note parsers, text-provider configuration and calls (the `llmbroker` dependency), generic LLM jobs/review APIs and the echo-words-specific handoff receiver. Preserve data, migrations, generic validation/publication, immutable revisions, chunk reconciliation, media storage and speech adapters. Remove tests for discarded responsibilities and refresh generated API contracts.
- **Inbox:**
  - Item API: one phrase or word, optional audio, optional free-text context; stable sender identity and idempotent receipt. Intake credentials stay server-side.
  - Drop space: a + button in the top bar opens a panel over the current screen, pre-filled from the clipboard on that tap (`navigator.clipboard.readText()`; fall back to an empty box when it rejects), with a note field, Send and recent entry statuses and failure reasons. Browser tests cover pre-fill, rejection fallback, closing back mid-card and that nothing is sent without Send.
  - Durable entries with statuses: waiting, preparing, items added, failed with reason. Adding a note to a drop requeues it. Recreate card creates an entry with the item and the optional comment.
- **Agent runner:**
  - One drop at a time, a bounded run time and crash recovery through a durable claimed task with a lease, like the speech work.
  - Configurable CLI: Claude Code (`claude -p --chrome`, started with `ANTHROPIC_API_KEY` removed from the child environment so the claude.ai login is used) or Codex (`codex exec` with its browser tool, once verified). The working directory is the source-instructions folder.
  - Capture the outcome into the drop status; never prompt the learner. Tests fake the CLI boundary; no real agent runs in tests.
- **Agent access to Phrase Gym:** read the per-language card/help settings, upload media and submit prepared packages. Document the actual commands/endpoints in the guide.
- **Preparation guide:** rewrite for the [current card rules](../decision-targeted-drilling.md): fixed wording and slots, one preferred phrase per moment, separated speaker turns, prompts in the card language, one hint, familiar-material marking, confusable partners, help per language, accepted German variants, and what to create for a bare item from the API (situation, partner line, short dialogue where useful, help).
- **Source instructions folder:** configurable location outside the repository, initialized as a git repository, with starting instructions for EPAM course pages (find the audio and cut each sentence's clip without the spoken item number) and plain pasted text.
- **Checks before relying on unattended runs:**
  - Whether EPAM course pages and their audio need the learner's login.
  - Codex non-interactive browser access to signed-in pages.
  - Approving the course sites for the agent in the browser extension.
- **Assessment:** run representative drops (an EPAM page with numbered audio, echo-words words in English and German, pasted movie lines, a book page) and held-out examples: short replies, openers sharing first words, long phrases, near-duplicates across lessons, idioms, frames, missing context, German chunks and Serbian. Inspect semantic quality separately from structural conformance. Model-result measurements need a planned fixture set and a fresh reviewer before the results are used for real imports.

External sources remain read-only for the agent unless separately authorized.

## echo-words outbox

Work in the echo-words project, which needs its own authorization: export each new word to the Phrase Gym inbox instead of Anki. Keep unsent words in an outbox file on the echo-words server, send on each new word and periodically, and remove only the words the inbox confirmed. Send the word and its audio through the item API, with the chosen sense and source sentence as context.

## Prompt and practice redesign

Implements the [prompts, words, help and practice decisions](../decision-targeted-drilling.md) and the [learning policy](../decision-core-trainer.md).

- **Prompts:** replace gapped-phrase cues with prompts (situation in the card language, optional preceding line with its audio, one hint) across contracts, validation, publication, pinned revisions and generated client types. Re-prepare the local 44-phrase pack through the agent rather than converting its gapped cues.
- **Card:**
  - Show the situation, and play and show the preceding line once. Show no phrase text before the response.
  - Hint shows the one cue, then the full answer, and marks the response supported. A hint on a target with a memory state records Not yet.
  - Saying the phrase with the reference ends the comparison only after a wrong or effortful attempt.
  - Delete and Recreate card replace the unclear-cue report. Delete retires the target from practice and lookup, keeping history and recordings. Recreate card takes an optional comment and creates an inbox entry. Either one before rating leaves the attempt ungraded.
  - Browser tests cover hint support and grading, rotation, the conditional say-along and that no phrase text appears before the response.
- **First meeting:** material marked as already studied starts with an unaided attempt. New material plays the reference, shows the phrase with its situation and asks for one say-along, with no grade; its first recall comes after the existing repair spacing (five minutes and two other finalized targets) or in a later session.
- **Card language and help per language:**
  - Replace the single native-language explanation source with a card language, a help language and a translation-with-answer choice per learning language. English: English cards and help, translation off. German: Russian cards and help, translation on, shown on reveal and after comparison. Serbian: Serbian cards; its help language is still open.
  - Keep the UI-language preference and migrate existing settings.
- **Word items:**
  - One schedule built around saying; understanding is part of the first meeting.
  - English and Serbian: missing-word context, whole-sentence answer, definition and contrast note. German: Russian meaning of a chunk, chunk answer with accepted variants, gloss with article/plural on tap.
  - A new confusable partner waits until its partner has been recalled once on a later day.
  - Ask the user for Serbian's help language before preparing Serbian words.
- **Game elements:**
  - Dialogue runs: no timer; partner lines play, the learner says each own line, the reference plays and the rating continues the run. Lines said without visible text are ordinary reviews; after the full script they are supported.
  - Lesson progress shows phrases recalled without help at their last check and the overdue count.
  - Real calls: choose two or three phrases, then mark each used, no chance or forgot, with no schedule effect.
  - A weekly target.
- **Vault course phrases:** once the item API exists, an external agent session reads the learner's 63 English phrases in the four lessons of `languages/english/фразы` (read-only) and sends each phrase with its audio and its dialogue line as context. Verify the prepared cards, media and playback in the app.

## Device acceptance

Complete the [device gaps](../open-problems.md) on actual iPhone Safari over trusted HTTPS, with speaker and headphones. Check longer/successive takes, saved replay, interruption/reopening, temporary server failure/retry and accessibility. Record only actionable gaps and current acceptance limits, without accumulating review logs in specs.

## Private release and recovery

- Measure the built app, speech work and agent runs on the laptop: idle/API/media peaks, agent run memory and disk growth.
- Verify private access to the laptop from the iPhone and from the echo-words server, and the behavior while the laptop sleeps. Deployment changes require an explicit user request.
- Demonstrate a consistent database/media backup and isolated restore, including every language, settings, schedules/history, every learner take, reference provenance, inbox drops and unfinished audio or agent work. Include the source-instructions folder in backups. Rehearse upgrade and rollback using matching snapshots where needed.
- Prepare operational documentation covering configuration, start/stop/update, the agent CLI login, health checks, storage/logging and recovery. Keep secrets and personal media outside Git.

## Validation

Use behavioral tests for remaining implementation changes, then run `uv run inv pre` followed by `uv run inv test`, including frontend and both browser engines. Speech/content, agent output, actual-device and host recovery acceptance stay separate from that software gate. No learning-effectiveness experiment, extra general review loop, push or deployment follows from this plan.
