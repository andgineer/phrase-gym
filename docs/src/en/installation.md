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
  login and settings, whatever model you use elsewhere: Claude Haiku 5.5 writes each card and a
  separate Claude Sonnet 5 session checks it, both at low effort. Coach never uses an API key.
  The model gets only a file-reading
  tool that works inside the current preparation's folder; reading `.env` files, your home
  directory and your learning data is refused. Read-allow rules, tools and servers you add in
  your own Claude settings can widen that restriction; Coach does not isolate them.
- A local English [Piper voice](piper-voice.md) for the reference audio.

Run Coach in another terminal:

```bash
PHRASE_GYM_PIPER_CONFIG=/absolute/path/to/piper-voice.json phrase-gym coach
```

For a source checkout, use `uv run phrase-gym coach`. Coach receives typed text through Phrase
Gym's API, has one session write the card and a second session check your text and every example,
meaning cue and hint the first one wrote, then generates the reference audio and publishes the
card. Anything the check doubts is left out; if no checked example remains, you get the basic
card described below for the Gemini route. When the two sessions disagree about what you meant,
the **Coach** page lists the text with a reason instead of adding a card. Coach prints a line for
every preparation that fails, with the folder that keeps that attempt's prompts and answers. It
also keeps a small private record of each request's models, time and reported token usage in
`.phrase-gym-coach/usage/`: inside the source checkout, or in a private cache folder for an
installed Coach.
Imports from an external AI agent arrive with complete audio and do not need Coach. Piper uses
the separately pinned private environment described in the voice setup guide.

To prepare with the [Codex CLI](https://developers.openai.com/codex/cli) instead, log in to it
with `codex login` and start Coach with `phrase-gym coach --cli codex`. Coach uses the same card rules for each CLI and keeps the CLI you select. Commands from the
Codex model run in a restricted sandbox that can read only the current preparation's files and
cannot read `.env` files, your home directory or your learning data, write files or reach the
network; tools and servers from your own Codex settings are not covered. On Codex, GPT-5.6 Luna
writes each card and a separate GPT-5.6 Terra session checks it, both at low effort.

Without a Claude or Codex subscription, `phrase-gym coach --cli gemini` uses the free tier of
the [Antigravity CLI](https://antigravity.google/) (`agy`, logged in once with a Google account)
on Gemini 3.8 Flash at low effort. On this route Coach only checks your text: it confirms the
word or phrase and fixes an obvious typo. You then get a basic card that asks you to fill the gap
in your own sentence, or to recall the word from a partial form such as **def…ly**, without an
explanation or extra examples. Coach runs `agy` the way news-recap does: one launch at a time, at
least ten seconds apart, with its permission prompts skipped, so the model's tools are not
limited to the preparation folder. If Antigravity renames the model, Coach switches to the
cheapest listed Flash model for the rest of its run. Free-tier usage is subject to a quota.

The `install-coach`, `uninstall-coach`, and `deploy` commands report that they are unavailable.
The optional `coach` and `deploy` extras are reserved for future preparation and provisioning
features. The core server and help need only the lean installation.

For access from another device, see [private Tailscale access](private-access.md).
Keep a private [backup of your learning data](backup-and-restore.md).
