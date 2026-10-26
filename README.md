# Phrase Gym

Phrase Gym is a small local Python/Vue application for studying useful English phrases and
keeping private learner recordings. This checkpoint supports package import, ungraded first
study, reference playback, optional recording, comparison, and saved replay. Later recall and
scheduling remain the next stage.

## Install and launch

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
uv run python scripts/checks.py pre
uv run python scripts/checks.py test
```

The second command runs Python tests, client typechecking/unit/build checks, and silent real-backend
browser tests in Chromium and WebKit. Install browser binaries from `frontend/` with
`npx playwright install chromium webkit`. Install the commit hook once with
`git config core.hooksPath .githooks`.

Current decisions and remaining work start at [specs/START-HERE.md](specs/START-HERE.md).
