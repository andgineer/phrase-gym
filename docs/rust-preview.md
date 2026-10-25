# Rust preview access

This page describes the currently runnable Rust prototype. The [target architecture](../specs/architecture.md) is Python plus the retained Vue GUI; do not treat the commands or profile protocol here as replacement requirements.

Prototype Server, Coach and browser run directly on the same MacBook. They use no application authentication, tokens, credential files, login cookies or OS credential store. Remote operation and social sign-in are outside the new delivery ladder.

Build the retained client once with `npm --prefix frontend run build`. After building Coach with `cargo xtask coach-bundle`, use these two commands in separate terminals on macOS:

```sh
cargo xtask run --data-dir /private/tmp/phrase-gym-preview
open "target/coach-bundle/Phrase Gym Coach.app"
```

Open `http://127.0.0.1:8000/learn/en`. Server and Coach share port 8000 by default; no Connect action or manual Server configuration is needed. The equivalent binary command is `phrase-gym-server serve --data-dir /private/tmp/phrase-gym-preview --client-dir frontend/dist`.

If port 8000 is occupied, use `cargo xtask run --port 8181 --data-dir /private/tmp/phrase-gym-preview` and save `http://127.0.0.1:8181` under Coach's **Server connection** settings. This optional nonsecret override is only for another local port. Pending input retains its original Server identity when settings change.

Server always listens on `127.0.0.1` and rejects non-loopback actual socket peers. Host and mutation-Origin checks defend browser requests; forwarded headers never establish socket identity. The host machine and local processes are trusted. Do not expose this unauthenticated demo through Funnel, a public listener, another proxy or a port forward.

To practise on your own iPhone, serve the Server privately on your tailnet and pass that exact origin; the local address keeps working at the same time:

```sh
tailscale serve --bg --https=443 http://127.0.0.1:18487
phrase-gym-server serve --data-dir "$DATA_DIR" --client-dir frontend/dist --port 18487 --tailscale-origin https://machine.tailnet.ts.net
```

Open `https://machine.tailnet.ts.net` on the iPhone with Tailscale connected. Server refuses Funnel requests and any other host or origin.

The initial profile records the [specified learner preferences](../specs/decision-personal-preferences.md). Profile mutation/restore remains an explicit operator workflow using stable mutation IDs and expected revisions. Preparation stages bounded input/profile/guide files and disables command network access. The selected normal Codex home and repository may also supply instructions; keep Server stores and recordings outside the permitted workspace, and deny the repository `.env` to commands as described in Coach setup. Local trusted operator API clients share access; the new local MVP requires neither fine-grained application identity nor social sign-in.

The generated Rust contract includes `PreparedEnglishLesson` for one lesson of one to five English targets (fixed phrases, reusable frames and contextual word senses). It pins a personal-profile revision plus guide and assessment versions, groups stable source identities, supplies one or two situations, optional partner turns, one partial hint and verified supplied or generated reference media. The core validator checks structure, identity/revision preconditions, reference-byte integrity and provenance and exact role/text/language bindings. It rejects gaps outside word-context tasks, exposed selected answers, mismatched context/reference bindings, invalid frame slots and unresolved English preferences. These checks do not certify semantic prompt quality or language usage. The production English inbox and fenced atomic publication path is implemented. These commands establish Server access. The [bounded Coach text demo](coach-desktop.md#local-english-lesson-demo) uses the selected Codex and generated English speech; the [delivery ladder](../specs/plan/implementation-plan.md) defines the smaller replacement and its own acceptance.
