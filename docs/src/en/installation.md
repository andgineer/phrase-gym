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

## Preparation with Coach

Coach is the application's separate preparation process for text typed into the **Coach** field.
It needs two things on the same Mac:

- The [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code), installed and logged in
  once with your Claude subscription (run `claude` and log in). Coach uses your normal Claude
  login, default model and settings and never uses an API key. The model gets only a file-reading
  tool that works inside the current preparation's folder; reading `.env` files, your home
  directory and your learning data is refused. Read-allow rules, tools and servers you add in
  your own Claude settings can widen that restriction; Coach does not isolate them.
- A local English [Piper voice](https://andgineer.github.io/phrase-gym/piper-voice/) for the reference audio.

Run Coach in another terminal:

```bash
PHRASE_GYM_PIPER_CONFIG=/absolute/path/to/piper-voice.json phrase-gym coach
```

For a source checkout, use `uv run phrase-gym coach`. Coach receives typed text through Phrase
Gym's API, prepares a card with Claude in separate preparation and review sessions, generates its
reference audio and publishes it. A card usually takes a minute or two. Coach prints a line for
every preparation that fails, with the folder that keeps that attempt's prompts and answers.
Imports from an external AI agent arrive with complete audio and do not need Coach. Piper uses
the separately pinned private environment described in the voice setup guide.

To prepare with the [Codex CLI](https://developers.openai.com/codex/cli) instead, log in to it
with `codex login` and start Coach with `phrase-gym coach --cli codex`. Codex is not yet
qualified for Coach; Coach never switches between the two on its own. Each CLI keeps its own
model and settings, and Phrase Gym does not choose a model.

The `install-coach`, `uninstall-coach`, and `deploy` commands report that they are unavailable.
The optional `coach` and `deploy` extras are reserved for future preparation and provisioning
features. The core server and help need only the lean installation.

For access from another device, see [private Tailscale access](https://andgineer.github.io/phrase-gym/private-access/).
Keep a private [backup of your learning data](https://andgineer.github.io/phrase-gym/backup-and-restore/).
