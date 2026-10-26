# Phrase Gym

[![CI](https://github.com/andgineer/phrase-gym/workflows/CI/badge.svg)](https://github.com/andgineer/phrase-gym/actions)
[![Coverage](https://raw.githubusercontent.com/andgineer/phrase-gym/python-coverage-comment-action-data/badge.svg)](https://htmlpreview.github.io/?https://github.com/andgineer/phrase-gym/blob/python-coverage-comment-action-data/htmlcov/index.html)

Phrase Gym is a small local Python/Vue application for studying useful English phrases and
keeping private learner recordings. This checkpoint supports package import, ungraded first
study, reference playback, optional recording, comparison, and saved replay. Later recall and
scheduling remain the next stage.

## Install and launch

The server runs on macOS and Linux. Use a macOS browser for the MVP media player and recording;
Linux browser audio support is planned after the MVP.

Requirements: Python 3.13, [uv](https://docs.astral.sh/uv/), Node 22.12+ and a configured English
Piper voice. Install the pinned environments and build the client once:

```bash
uv sync --frozen
npm --prefix frontend ci
npm --prefix frontend run build
```

Set `PHRASE_GYM_PIPER_CONFIG` to a JSON voice configuration described in
[the import guide](docs/importing-phrases.md#piper-voice), then launch:

```bash
PHRASE_GYM_PIPER_CONFIG=/absolute/path/to/piper-voice.json uv run phrase-gym
```

Open <http://127.0.0.1:8000>. The app listens only on loopback and stores its database,
references, and recordings in the operating system's Phrase Gym application-data directory.
For a separate data directory, add `--data-dir /absolute/path`.

## Add and study material

Open **Add material**, paste a package or choose its UTF-8 JSON file, and select **Import**.
The app validates the whole package, prepares every reference through Piper, and adds all cards
together. A preparation error keeps the submitted package for explicit retry. Exact package
retries return the existing result; a changed package ID or repeated source/item identity is
rejected without overwriting learned material.

Study shows the full phrase and meaning/use help. Reference playback and recording are optional.
Every stopped take is retained separately and saved automatically; pending uploads remain in the
browser for retry. **Next** records ungraded study completion. The finish screen deliberately does
not invent a recall grade or schedule before the later-recall stage exists.

See [the package schema, example, and external-agent instruction](docs/importing-phrases.md).

## Development

```bash
. ./activate.sh
uv run pre-commit install
inv --list
uv run python scripts/checks.py pre
uv run python scripts/checks.py test
```

`scripts/checks.py test` runs Python tests, client typechecking/unit/build checks, and silent real-backend
browser tests in Chromium and WebKit on macOS. CI runs the full gate on macOS and separately
runs all Python server tests on Linux. For server-only iteration, use `uv run pytest`.
Install browser binaries from `frontend/` with
`npx playwright install chromium webkit`. If migrating from the temporary custom hook, run
`git config --unset core.hooksPath` before `pre-commit install`.

The scaffold's `inv pre`, `inv reqs`, `inv docs-en`, `inv build-docs`, `inv version`, and
`inv ver-bug` / `inv ver-feature` / `inv ver-release` commands are available.
`./build.sh` builds the client and Python distributions. A version command checks and commits the
version update, then pushes its release tag; the tag workflow publishes to PyPI only after CI passes.
See [development and release operations](docs/development.md) for setup and dry-run commands.

[Documentation](https://andgineer.github.io/phrase-gym/),
[Allure reports](https://andgineer.github.io/phrase-gym/builds/tests/),
[Codecov](https://app.codecov.io/gh/andgineer/phrase-gym), and
[Coveralls](https://coveralls.io/github/andgineer/phrase-gym) retain the scaffold's reporting links.

Created using [cookiecutter-python-package](https://github.com/andgineer/cookiecutter-python-package).

Current decisions and remaining work start at [specs/START-HERE.md](specs/START-HERE.md).
