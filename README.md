# Phrase Gym

[![CI](https://github.com/andgineer/phrase-gym/workflows/CI/badge.svg)](https://github.com/andgineer/phrase-gym/actions)
[![Coverage](https://raw.githubusercontent.com/andgineer/phrase-gym/python-coverage-comment-action-data/badge.svg)](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

Phrase Gym is a small local Python/Vue application for studying useful English phrases and
keeping private learner recordings. It supports package import, Daily and Extra practice with first
study and later spoken recall, phrase-specific Practice, self-rating, scheduled return, reference
playback, comparison, saved replay, and today's practice facts with a factual streak.

## Install and launch

The server runs on macOS and Linux. Use a macOS browser for the MVP media player and recording;
Linux browser audio support is planned after the MVP.

For a source checkout, use Python 3.13, [uv](https://docs.astral.sh/uv/), Node 22.12+ and a
configured English Piper voice for Coach synthesis. Install the pinned environments and build the client once:

```bash
uv sync --frozen
npm --prefix frontend ci
npm --prefix frontend run build
```

Start Phrase Gym without any Piper installation:

```bash
uv run phrase-gym run
```

For private iPhone access on your existing tailnet, use `uv run inv run --build --tailscale`. This points Tailscale Serve at the new server
and allows its exact HTTPS machine hostname. Open the HTTPS URL printed by Tailscale on the
iPhone with Tailscale connected.

The built wheel contains the Vue client. A lean installed package runs `phrase-gym run`.
Coach runs separately: set `PHRASE_GYM_PIPER_CONFIG` to the private voice file in
[the import guide](docs/importing-phrases.md#piper-voice), then run
`uv run phrase-gym coach` in another terminal. The private Piper environment belongs to Coach;
the server needs only core dependencies. Service registration and deployment commands remain
unavailable.

Open <http://127.0.0.1:8000> on the Mac. The app listens only on loopback and stores its database,
references, and recordings in the operating system's Phrase Gym application-data directory.
For a separate data directory, add `--data-dir /absolute/path`. Before the first release, a new
version may need a fresh data directory: startup then stops with an explanation and leaves the
earlier directory unchanged.

## Add and study material

Open **Add material**, paste a package or choose its UTF-8 JSON file, and select **Import**.
The app saves the request before acknowledging it. A text-only package stays pending while Coach
is offline; Coach prepares reference audio and returns it through the API. An external agent can
submit the same package with complete WAV references in one API request, without Coach. The server
validates and publishes all phrases, lessons and references together. A preparation error remains
visible for explicit retry. Exact package retries return the existing result; changed content under
a package ID or a repeated source/item identity is rejected without overwriting learned material.

The Practice entry offers **Search lessons**, **Lesson to drill** with its due and new counts,
**Daily**, **Extra**, and the Library with **Find a phrase**. Nothing starts until you press
**Daily**, which opens or resumes today's card. Daily offers the oldest due phrases first, up to 20 recalls, then up
to five new phrases per learning day; the stored limits are read live, so a raised limit lets Daily
continue. A new phrase opens on its answer with meaning help, reference playback and optional
recording; **Next** completes this first meeting without a rating. From the next learning day a due
phrase shows an answer-free situation cue with **New cue**, **Hint**, **Show** and **Details**.
After **Show**, you can still record a rehearsal, then rate the response you gave before help or
reveal: **Not yet**, **Effort** or **Got it**. A rating or **Skip** saves and advances at once.
**Delete** on a revealed card asks for confirmation and advances without a grade.

**Extra** starts a fresh pass for more practice today. Once today's completed recalls or first
meetings reach a daily limit, each press extends that category by one more limit (for example 20
recalls become 40); pressing again without further practice adds nothing, and completed practice is
never reset. Skipped phrases from earlier passes come back; phrases that are not yet due never do.
If nothing is waiting, Extra says so at once. **Stop** on an Extra card finishes any recording and
returns to the entry; **Daily** resumes that card later.

When a Daily or Extra pass runs out, the entry says so and shows today's practice above its
controls, across all lessons: completions, first meetings, Skips and ratings from Daily, Extra and
Practice; recordings saved today, with takes still uploading from this device counted separately;
**See today's phrases** with **More**, which keeps the wording of phrases deleted since; the
current and best practice streak with the last seven days; your cumulative work; and the next date
phrases are due. Any first meeting, rating or Skip counts the day toward the streak. Until today's
first practice the streak still shows the run that ended yesterday; after a missed day it is zero,
while the personal best and cumulative work remain. An entry left open refreshes when you return to
it, so it never shows yesterday as today.

Every stopped take is retained in the browser and uploads in the background with the cue, hint and
answer it was recorded with; pending uploads remain for retry and never block the next card.
**▶ Mine** and **Compare** replay the latest take of the open card.

The Library lists the selected lesson's phrases and searches all lessons by wording, meaning,
situation or use. Phrase details show the answer, what it is used for, the lesson and source,
reference playback, the next due date and saved recordings with their mode, cue and answer at the
time. Looking a phrase up never changes its schedule. **Practice** in details opens that phrase on
the same card, as a first meeting if it is new, and returns to its details after a rating, **Skip**
or **Next**. It does not use the daily limits, but its first rating on a learning day reschedules the
phrase even before it is due.
**Delete**, in the list or details, permanently removes a phrase with its reference audio and
saved recordings after confirmation; completed practice keeps its wording. A recording still
waiting to upload for that phrase stays on the device until you discard it. If deleted files cannot
be removed, the app keeps showing the problem with **Retry cleanup** until removal succeeds.
**Settings** contains the native language and learning timezone.

For a private stopped-app backup and restore, follow [the maintenance procedure](docs/backup-and-restore.md).

See [the package schema, example, and external-agent instruction](docs/importing-phrases.md).

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
See [development and release operations](docs/development.md) for setup and dry-run commands.

Use `uv run inv run --build` to rebuild the frontend and start the server, or `uv run inv run`
to start it with the existing client build.

[Documentation](https://andgineer.github.io/phrase-gym/),
[Allure reports](https://andgineer.github.io/phrase-gym/builds/tests/),
[Codecov](https://app.codecov.io/gh/andgineer/phrase-gym), and
[Coveralls](https://coveralls.io/github/andgineer/phrase-gym) retain the scaffold's reporting links.

Created using [cookiecutter-python-package](https://github.com/andgineer/cookiecutter-python-package).

Current decisions and remaining work start at [specs/START-HERE.md](specs/START-HERE.md).
