# Phrase Gym

[![CI](https://github.com/andgineer/phrase-gym/workflows/CI/badge.svg)](https://github.com/andgineer/phrase-gym/actions)
[![Coverage](https://raw.githubusercontent.com/andgineer/phrase-gym/python-coverage-comment-action-data/badge.svg)](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

Phrase Gym is a small local Python/Vue application for studying useful English phrases and
keeping private learner recordings. It supports package import, first study, later spoken recall,
self-rating, scheduled return, reference playback, comparison, and saved replay.

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
For a separate data directory, add `--data-dir /absolute/path`.

## Add and study material

Open **Add material**, paste a package or choose its UTF-8 JSON file, and select **Import**.
The app saves the request before acknowledging it. A text-only package stays pending while Coach
is offline; Coach prepares reference audio and returns it through the API. An external agent can
submit the same package with complete WAV references in one API request, without Coach. The server
validates and publishes all cards and references together. A preparation error remains visible
for explicit retry. Exact package
retries return the existing result; a changed package ID or repeated source/item identity is
rejected without overwriting learned material.

Study shows the full phrase and meaning/use help. Reference playback and recording are optional.
Every stopped take is retained separately and saved automatically; pending uploads remain in the
browser for retry. **Next** records ungraded study completion. On the next learning day, due cards
show an answer-free cue. Try to recall the wording, reveal it, then rate the original response:
**Not yet**, **With effort**, or **Recalled**. A difficult response can be repeated once with
the answer visible. **Study / skip grading** and **Unclear card** avoid a false forgetting grade.
The **Library** offers saved takes, dated reviews, Suspend/Restore, and ungraded practice.
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
