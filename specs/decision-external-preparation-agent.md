# Preparation inbox, agent and reference speech

## Ownership boundary

Material reaches Phrase Gym without manual preparation steps, through an inbox with two entrances: an item API and a drop space. A preparation agent turns every inbox entry into prepared material and submits it through the one generic intake. Phrase Gym starts the agent but contains no prompt writing, text-model calls or source parsing of its own; knowledge of sources lives only in the agent's instructions. Intake validates structure and bindings and publishes atomically. There is no human approval step: language quality is the agent's responsibility, and the learner deletes or recreates bad cards during practice.

## Inbox

**Item API.** A sender submits one phrase or word with optional audio and an optional free-text context, such as the chosen meaning or the sentence where the word was met. Everything else is the inbox's work: situation and partner line, a short dialogue where useful, translation and help according to the language settings, and audio where none was supplied. echo-words and external agents use this entrance. A complex one-off import, such as the course phrases and their audio in the learner's vault, is done by an external agent session that reads the source and sends items through this API; collecting them is outside Phrase Gym.

**Drop space.** The learner pastes anything: a phrase, a word, a link or a page of text, optionally with a note in plain words such as "cut the item number off the audio". The agent works out what to do from its source instructions, or does what seems reasonable for an unknown kind of drop.

The drop space opens from a + button in the top bar on every screen, including mid-card, as a panel over the current screen that closes back to the same place. Tapping + fills the box from the clipboard; Safari shows its own Paste button first when the text came from another app, Chrome asks permission once, and an unreadable or empty clipboard leaves the box empty for manual pasting. The learner checks the text, can add a note and taps Send. Nothing is sent without that check, so stray clipboard content such as a password never reaches the agent. The same panel shows the status of recent entries.

Every entry is stored unchanged with a stable identity and shows its status: waiting, preparing, the number of items added, or failed with the reason. A resent entry with the same identity is recognized, not duplicated. Adding a note to a drop processes it again. A card the learner asks to recreate returns to the agent as an inbox entry, with the learner's comment if one was given.

## Preparation agent

Phrase Gym starts a coding-agent CLI non-interactively for each waiting entry, one at a time, with a bounded run time. Claude Code and Codex are both supported, and the choice is configuration. The agent runs on the laptop under the learner's own subscription login.

The agent never asks the learner anything. Material it cannot prepare well is submitted for study and lookup only. When a whole entry fails, the reason is visible in the inbox. The agent does not compare new material with the learner's existing phrases, which would cost tokens on every run; the learner deletes duplicates from the card.

Pages behind a login are opened in the learner's local Chrome, where course sites are already signed in. Unattended runs need the agent's browser access approved for those sites in advance. Measured on 2026-10-15: Claude Code run non-interactively reached the local Chrome through the Claude in Chrome extension only when signed in with the claude.ai account; with an API key the browser tools did not load. Codex run non-interactively lists a browser tool supplied by the ChatGPT desktop app; whether it reaches signed-in pages is not yet verified.

## Instructions

The agent is configured in plain language, not through settings, in two layers:

- **Preparation guide**, owned by Phrase Gym and versioned with the intake format. It covers complete phrases and selected meanings, fixed wording and variable slots, one preferred phrase per conversational moment within the material at hand, prompts in the card language, one hint per phrase, marking material the learner has already studied, confusable partners, help in each language's help language only where useful, searchable concepts and dialogues with speaker turns separated. The rules themselves are in [prompts, words and help](decision-targeted-drilling.md). The agent reads the per-language card and help settings from Phrase Gym before preparing. It does not invent missing meanings.
- **Source instructions**, owned by the learner: one per kind of drop (EPAM course pages, plain pasted text, subtitles and so on) plus the agent's own working notes. They live in a separate version-controlled folder outside the code repository. Source-specific work belongs here, for example finding a course page's audio and cutting each sentence's clip without its spoken item number.

The learner changes behavior by saying it: in a note on one drop, or in a conversation with the agent in the instructions folder ("from now on, for EPAM pages…"). The agent edits its own instructions, and every change is a commit that can be read and undone. When the agent works out how to handle a new kind of drop, it saves that as a new source instruction.

## echo-words

echo-words exports each new word to the item API instead of Anki: the word, its audio and, as context, the chosen sense and the sentence where the learner met it. It keeps unsent words in an outbox file on its server and sends them on each new word and periodically. It removes only the words the inbox confirmed, so a word added during a send is not lost. This change is made in the echo-words project.

## Hosting

Phrase Gym, its inbox and the agent run on the learner's laptop. echo-words reaches the inbox over the private network. While the laptop sleeps, nothing is prepared and [practice is unavailable](decision-online-only.md); echo-words keeps new words in its outbox until the laptop is back.

## Intake and preservation

Stable source/submission identifiers make retries idempotent. Compatible edits preserve targets, schedules, recordings and unchanged chunk evidence; changed meanings require distinct targets. Pinned sessions keep their content. Source ordering and similar wording are not identities.

Optional supplied reference audio needs verified uploaded bytes and exact text/language/role/provenance. Preserve course/actor originals. Reject invalid or supplied-but-untransferred audio rather than treating it as a request for synthesis. Do not fetch arbitrary media URLs. Structural validation and content publication remain atomic internal responsibilities.

## Missing reference speech

Generate genuinely missing whole-phrase/turn audio inside Phrase Gym before practice, cache it and preserve exact speech input, engine/voice, locale/accent, settings and preparation version in its identity. Use one configured primary provider/voice per learning language and a fallback preserving that language and intended locale/accent. Provider failure has bounded fallback/retry and cannot break already prepared practice. Focused durable audio work does not justify a generic text-LLM pipeline or speech grading.

Piper Lessac for US English, Piper Thorsten for German and Edge Sophie for Serbian are starting candidates, not a quality ranking. No macOS-only speech dependency. Serbian speech input may need Cyrillic conversion while displayed text stays unchanged; names, digraph boundaries and foreign words need checking. The Piper serbski_institut voice is Lower Sorbian and must not be selected as Serbian merely from its label. Supplied actor clips always take priority.

Actual speech listening and the agent's language quality are distinct from schema/container tests. Choose voices using language-specific quality evidence. Private learner recordings are never provider inputs.
