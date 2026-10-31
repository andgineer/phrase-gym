# Installation and launch

Use Python 3.13 and [uv](https://docs.astral.sh/uv/getting-started/installation/).
Published packages include the built browser client:

```bash
uv tool install phrase-gym --upgrade --python 3.13
```

Until a release is published, install from a source checkout with Node 22.12+ on an LTS release:

```bash
uv sync --frozen
npm --prefix frontend ci
npm --prefix frontend run build
```

Start the server; Piper is not required to launch it:

```bash
phrase-gym run
```

For a source checkout, use `uv run phrase-gym run` instead. Open <http://127.0.0.1:8000>.
Use `phrase-gym run --help` for the port and data-directory options. Keep voice files and learner
recordings outside the source checkout. Actual microphone and headset acceptance requires
attended testing on the learner's MacBook.

## Audio preparation with Coach

Coach is the application's separate preparation process. For text-only imports, configure a local English [Piper voice](piper-voice.md)
and run Coach in another terminal:

```bash
PHRASE_GYM_PIPER_CONFIG=/absolute/path/to/piper-voice.json phrase-gym coach
```

For a source checkout, use `uv run phrase-gym coach`. Coach receives pending imports through
Phrase Gym's API and generates reference audio. Imports with complete reference audio do not
need Coach. Piper uses the separately pinned private environment described in the voice setup guide.

The `install-coach`, `uninstall-coach`, and `deploy` commands report that they are unavailable.
The optional `coach` and `deploy` extras are reserved for future preparation and provisioning
features. The core server and help need only the lean installation.

For access from another device, see [private Tailscale access](private-access.md).
Keep a private [backup of your learning data](backup-and-restore.md).
