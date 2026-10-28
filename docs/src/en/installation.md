# Installation and launch

Use Python 3.13 and [uv](https://docs.astral.sh/uv/getting-started/installation/).
Published packages include the built browser client:

```bash
uv tool install phrase-gym --upgrade --python 3.13
```

Until a release is published, install from a source checkout:

```bash
uv sync --frozen
npm --prefix frontend ci
npm --prefix frontend run build
```

Configure a local English [Piper voice](importing-phrases.md#piper-voice), then start the server:

```bash
PHRASE_GYM_PIPER_CONFIG=/absolute/path/to/piper-voice.json phrase-gym run
```

For a source checkout, use `uv run phrase-gym run` instead. Open <http://127.0.0.1:8000>.
Use `phrase-gym run --help` for the port and data-directory options. Keep voice files and learner
recordings outside the source checkout. Actual microphone and headset acceptance requires
attended testing on the learner's MacBook.

The `coach` command is present but preparation is pending the API handoff. The
`install-coach`, `uninstall-coach`, and `deploy` commands report that they are unavailable.
The `coach` extra supplies Piper; `deploy` is reserved for later provisioning. The core server
and help need only the lean installation.
