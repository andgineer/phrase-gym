# Phrase Gym

[![CI](https://github.com/andgineer/phrase-gym/workflows/CI/badge.svg)](https://github.com/andgineer/phrase-gym/actions)
[![Coverage](https://raw.githubusercontent.com/andgineer/phrase-gym/python-coverage-comment-action-data/badge.svg)](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

Phrase Gym is a small local Python/Vue application for studying useful English phrases and
keeping private learner recordings. It supports typed phrases and words prepared by Coach, external-agent batches, Daily and Extra practice with spoken
recall from the first attempt, phrase-specific Practice, self-rating, scheduled return, reference
playback, comparison, saved replay, and today's practice facts with a factual streak and an
optional weekly target.

## Install and launch

The server runs on macOS and Linux. Use a macOS browser for the MVP media player and recording;
Linux browser audio support is planned after the MVP.

Follow the [installation guide](https://andgineer.github.io/phrase-gym/installation/) to install and launch the application,
including its optional material-preparation process. For another device, see
[private access](https://andgineer.github.io/phrase-gym/private-access/).

Open <http://127.0.0.1:8000> on the Mac. The app listens only on loopback and stores its database,
references, and recordings in the operating system's Phrase Gym application-data directory.
For a separate data directory, add `--data-dir /absolute/path`. A new version keeps your data
and upgrades the database in place. If startup finds a database from a newer or unknown version,
it stops with an explanation and leaves the directory unchanged.

## Add and study material

Type a phrase or word, optionally followed by the sentence it came from, into the field at the
top right and press **Coach**. The application's Coach prepares a complete card with reference
audio using your logged-in Claude Code CLI and local Piper voice, and adds it to the **Added with
Coach** lesson. `phrase-gym coach --cli codex` uses the logged-in Codex CLI instead; Coach never
switches between the two on its own. An external AI agent can instead prepare a course batch with its reference audio
and submit it for immediate study. See [how to add phrases](https://andgineer.github.io/phrase-gym/importing-phrases/) for
both workflows.

The Practice entry offers a lesson field (empty means **All lessons**; type in it to narrow the
list) with its due and new counts, **Daily**, **Extra**, and the Library with **Find a phrase**. Nothing starts until you press
**Daily**, which opens or resumes today's card. Daily takes the longest-due phrases up to your daily due limit and randomly chosen new
phrases up to your daily new limit (20 and five by default, both editable in **Settings**),
then offers them mixed in random order; the stored limits are read live, so a raised limit lets Daily
continue. Every phrase starts with an answer-free situation cue, optional recording, **New cue**,
**Hint**, **Show** and **Details**, including a new phrase. If you do not know it, reveal and choose
**Again**; the first rating initializes its schedule immediately.
After **Show**, you can still record a rehearsal, then rate the response you gave before help or
reveal: **Again**, **Hard**, **Good** or **Easy**. A rating or **Skip** saves and advances at once.
**Delete** on a revealed card asks for confirmation and advances without a grade.

**Extra** starts a fresh pass for more practice today. Once today's completed recalls or new
attempts reach a daily limit, each press extends that category by one more limit (for example 20
recalls become 40); pressing again without further practice adds nothing, and completed practice is
never reset. Skipped phrases from earlier passes come back; phrases that are not yet due never do.
If nothing is waiting, Extra says so at once. **Stop** on an Extra card finishes any recording and
returns to the entry; **Daily** resumes that card later.

When a Daily or Extra pass runs out, the entry says so and shows practice facts below its
controls, across all lessons: the current and best practice streak with the last seven days;
**Today** with its recalls and new phrases, ratings, Skips and recordings, with takes still
uploading from this device counted separately, and **See today's phrases** with **More**, which
keeps the wording of phrases deleted since; **Progress** with the days you practised and
everything done in them (with older ungraded first meetings labeled separately); and **Total**
with how many phrases are due now, due later and new, plus how many are in long-term memory.
Any rating or Skip counts the day toward the streak. Until today's
first practice the streak still shows the run that ended yesterday; after a missed day it is zero,
while the personal best and your progress remain. If you choose a weekly target in **Settings**,
the streak is joined by this week's practice days against it, such as **This week: 2 of 3 days**;
a week runs Monday to Sunday, counts days the same way as the streak and starts afresh each Monday
without carrying a shortfall forward. An entry left open refreshes when you return to
it, so it never shows yesterday as today.

Every stopped take is retained in the browser and uploads in the background with the cue, hint and
answer it was recorded with; pending uploads remain for retry and never block the next card.
**Mine** and **Compare** replay the latest take of the open card.

The Library lists the selected lesson's phrases and searches all lessons by wording, meaning,
situation or use. Phrase details show the answer, what it is used for, the lesson and source,
reference playback, the next due date and saved recordings with their dates and **Mine** replay
buttons. Looking a phrase up never changes its schedule. **Practice** in details opens that phrase on
the same hidden-answer card, including when new, and returns to its details after a rating or **Skip**.
It does not use the daily limits, but its first rating on a learning day reschedules the
phrase even before it is due.
**Delete**, in the list or details, permanently removes a phrase with its reference audio and
saved recordings after confirmation; completed practice keeps its wording. A recording still
waiting to upload for that phrase stays on the device until you discard it. If deleted files cannot
be removed, the app keeps showing the problem with **Retry cleanup** until removal succeeds.
**Settings** contains the native language, learning timezone, daily limits, how often phrases
come back (**Often**, **Balanced** or **Rarely**) and an optional weekly target of one to six
practice days, which you can change or pause at any time; each rating button shows when that grade
brings the phrase back. The weekly target never changes schedules, daily limits or which phrases
Daily offers.

For a private stopped-app backup and restore, follow [the maintenance procedure](https://andgineer.github.io/phrase-gym/backup-and-restore/).

## Development

```bash
. ./activate.sh
uv run pre-commit install
inv --list
uv run inv pre
uv run inv test
```

`inv test` runs Python tests, client typechecking/unit/build checks, and silent real-backend
browser tests in Chromium and WebKit on macOS. CI runs the full gate on macOS and separately
runs all Python server tests on Linux. For server-only iteration, use `uv run pytest`.
Install browser binaries from `frontend/` with
`npx playwright install chromium webkit`. If migrating from the temporary custom hook, run
`git config --unset core.hooksPath` before `pre-commit install`.

The scaffold's `inv pre`, `inv reqs`, `inv docs-en`, `inv build-docs`, `inv version`, and
`inv ver-bug` / `inv ver-feature` / `inv ver-release` commands are available.
`uv run inv build` builds the client and Python distributions. A version command checks and commits the
version update, then pushes its release tag; the tag workflow publishes to PyPI only after CI passes.
See [development and release operations](specs/development.md) for setup and dry-run commands.

Use `uv run inv run --build` to rebuild the frontend and start the server, or `uv run inv run`
to start it with the existing client build.

[Documentation](https://andgineer.github.io/phrase-gym/),
[Allure reports](https://andgineer.github.io/phrase-gym/builds/tests/),
[Codecov](https://app.codecov.io/gh/andgineer/phrase-gym), and
[Coveralls](https://coveralls.io/github/andgineer/phrase-gym) retain the scaffold's reporting links.

Created using [cookiecutter-python-package](https://github.com/andgineer/cookiecutter-python-package).

Current decisions and remaining work start at [specs/START-HERE.md](specs/START-HERE.md).
