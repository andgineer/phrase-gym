# Rust preview access

Server, Coach and browser run directly on the same MacBook. They use no application authentication, tokens, credential files, login cookies or OS credential store. Remote operation and social sign-in are later work.

Build the retained client once with `npm --prefix frontend run build`. After building Coach with `cargo xtask coach-bundle`, use these two commands in separate terminals on macOS:

```sh
cargo xtask run --data-dir /private/tmp/phrase-gym-preview
open "target/coach-bundle/Phrase Gym Coach.app"
```

Open `http://127.0.0.1:8000/learn/en`. Server and Coach share port 8000 by default; no Connect action or manual Server configuration is needed. The equivalent binary command is `phrase-gym-server serve --data-dir /private/tmp/phrase-gym-preview --client-dir frontend/dist`.

If port 8000 is occupied, use `cargo xtask run --port 8181 --data-dir /private/tmp/phrase-gym-preview` and save `http://127.0.0.1:8181` under Coach's **Server connection** settings. This optional nonsecret override is only for another local port. Pending input retains its original Server identity when settings change.

Server always listens on `127.0.0.1` and rejects non-loopback actual socket peers. Host and mutation-Origin checks defend browser requests; forwarded headers never establish socket identity. The host machine and local processes are trusted. Do not expose this unauthenticated demo through a proxy, public listener or port forward. It needs no network service or remote access setup.

The initial profile records the [specified learner preferences](../specs/decision-personal-preferences.md). Profile mutation/restore remains an explicit operator workflow using stable mutation IDs and expected revisions. Preparation model tools receive only bounded attempt files and cannot call profile or recording APIs. Local trusted operator API clients share access; fine-grained application identity is deferred with social sign-in.

The generated Rust contract includes `PreparedEnglishLesson` for one lesson of one to five short fixed English phrases. It pins a personal-profile revision plus guide and assessment versions, groups stable source identities, supplies one or two situations, optional partner turns, one partial hint and original reference media. The core validator checks structure, identity/revision preconditions, supplied-audio integrity and exact role/text/language bindings. It rejects gapped fields, complete/accepted answers in prompts or lesson titles, unresolved English preferences and unsupported kinds. These checks do not certify semantic prompt quality or language usage. The production English inbox and fenced atomic publication path is implemented. These commands establish Server access. Remaining Coach integration and generated-lesson acceptance are tracked in the [implementation plan](../specs/plan/implementation-plan.md#ordered-implementation-queue).
