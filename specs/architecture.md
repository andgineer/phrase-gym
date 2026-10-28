# Small personal application

## Application boundary

Phrase Gym is a lightweight local Python service serving the retained Vue frontend and an HTTP API, with SQLite and local audio storage. The browser is the GUI. Use the normal browser on the learner's MacBook; no Qt, native Coach window or Electron bundle is required. A launcher can open the browser later if starting the application proves inconvenient.

Retain the original Vue visual and interaction design while replacing backend workflows. The compact header, warm page, centered practice card, serif phrase heading, round microphone control and responsive spacing are established interface choices. Show only controls supported by the current Python API; later practice and library workflows can return within this interface when implemented.

The Python server must work on both macOS and Linux. The MVP media player and recording experience target browsers on macOS; Linux browser audio support is deferred until after the MVP. Server portability does not expand the local-only access boundary.

Phrase Gym owns cards, practice, scheduling, settings, recordings, the durable preparation request state and publication of ready content. A separately runnable, headless Python Coach owns reference synthesis, later agentic CLI calls and their heavier dependencies; it needs no special GUI. Phrase Gym owns the only user interface. Server startup must not depend on Piper, a model runtime or a preparation CLI subprocess. Already prepared practice remains available while Coach is stopped.

Python is chosen for straightforward local iteration and the Coach's speech/model integration. Use ordinary libraries for Phrase Gym's HTTP validation and scheduling and for Coach's speech work rather than maintaining ports of their internals. Keep the Phrase Gym service small even when Coach has large dependencies.

## Package and commands

Ship one `phrase-gym` Python distribution from one `pyproject.toml` and `uv.lock`. Keep all Python application source under `src/phrase_gym/server/`, `src/phrase_gym/coach/` and `src/phrase_gym/deploy/`. Put server HTTP adapters under `server/api/`; keep command parsing and dispatch small in `server/main.py`. Server flags, such as port, override configuration. Each module has one focused responsibility and follows the repository size boundary.

Core dependencies support the server alone. Piper runs in Coach's separately pinned private environment; the optional `coach` extra can supply later model preparation dependencies; the optional `deploy` extra supplies future cloud provisioning dependencies. The same wheel contains all three source areas, even in a lean installation. `uv tool install phrase-gym` installs the server; `uv tool install 'phrase-gym[coach,deploy]'` installs the full set. Installing only the `coach` extra is also supported. Server startup and CLI help must work with core dependencies alone; server runtime code must not depend on Coach or cloud packages. One installed environment is sufficient for the separate server and Coach processes; no second Coach distribution, environment or automatic dependency installer is required.

The intended user commands are `phrase-gym run` for the server, `phrase-gym coach` for foreground preparation, `phrase-gym install-coach` to register and start a per-user background Coach service, `phrase-gym uninstall-coach` to remove that registration while preserving application data, and `phrase-gym deploy` for a future configured cloud server. These commands describe the target interface, not current availability. `deploy/` owns local service registration and future cloud deployment, while Coach owns its runtime and the server owns practice and publication. Local service registration must not require the cloud `deploy` extra. The later service implementation targets macOS per-user launchd and Linux systemd user services. It does not move Coach into the server process or bypass the API-only handoff.

The distribution includes the built Vue client and required application resources. Installed user commands need no source clone, Node or Invoke. Invoke remains the repository's development, build, test and release entry point and may wrap packaged user operations. Keep the current documented development and installation commands accurate until the corresponding packaged commands are implemented.

## Responsibilities

Keep HTTP adapters, content validation/import, practice policy and persistence focused in Phrase Gym; put synthesis and later bounded AI execution in Coach. Vue renders state and owns browser audio effects. Generate client API types from Phrase Gym's Python contract. There is no need to preserve every prototype endpoint or module boundary.

All preparation submissions, whether from an external agent, a later echo-words export or Vue copy/paste, enter through the Phrase Gym API. Vue reads status and requests retries through that API as well; the browser never contacts Coach. Phrase Gym validates a work submission and persists it in SQLite before acknowledging it, even when Coach is offline. Coach retrieves pending work and submits results only through that API; it never reads or writes the application database directly. Phrase Gym validates results and publishes the entire ready batch, including references, in one transaction. Stable request/result IDs make retries idempotent across restarts and lost responses. An interrupted or failed item remains visible for explicit retry, while unrelated ready practice continues. Use the smallest database-backed handoff that provides these guarantees; no broker, general distributed worker framework or high-availability protocol is required.

Use one application database and data directory outside the model's workspace. Keep submitted work, original reference audio and learner takes with their card/attempt association. Ordinary SQLite transactions make a batch publication and a rated review coherent. Retrying a completed submission or review must not duplicate cards or count the same review twice. Phrase Gym remains the database writer; do not construct a general event platform.

Record the prompt, answer and support used for an attempt so later edits cannot misdescribe history. A replacement meaning starts a new card. Simple snapshots of those fields suffice; immutable profile registries, publication generations, cross-target dependency graphs and versioned job capabilities are not required.

## Growth after the MVP

Preserve stable card/source identities, attempt identities, recorded support and independent audio references. Keep scheduling policy separate from activity presentation and recording storage. Later [speaking activities](decision-transfer-and-engagement.md) can associate an attempt with several existing expressions without redefining a card or interpreting every recording as a memory review. Achievement and real-use records are separate from FSRS history.

Implement only the first MVP's records and APIs now. Add later activity records and optional content fields through ordinary schema migrations when their stages are built; no placeholder dialogue tables, generic workflow engine, plugin framework or speculative API endpoints are needed. Preserve existing MVP cards, grades and recordings through those upgrades. Version the import package format so later additions can coexist with the MVP format; unknown versions fail clearly, and compatible old packages remain importable.

## Local access and privacy

For the first versions, Phrase Gym and Coach run on the same machine. Listen on loopback and serve Vue and the API from the same origin. Retain basic Host/Origin checks against unrelated websites. There are no accounts, application credentials, synchronization or public endpoints in the MVP. The learner has authorized private iPhone access through Tailscale Serve. Keep the backend on loopback and opt in to one exact HTTPS machine origin; Tailscale controls which tailnet devices may reach the shared library. Do not use Funnel or a public listener. This adds private access, not per-user data separation or automatic iPhone audio qualification.

The planned later placement runs Phrase Gym on a cloud host and Coach on the learner's laptop. The API-only handoff permits that placement without sharing the database or moving Piper/model dependencies into Phrase Gym. Remote access, authentication and deployment details must be designed before implementation, and deployment still requires the learner's explicit request. The current loopback/Tailscale setup does not implement it.

External course extraction happens outside the app. Imported source links are attribution, never instructions to fetch or execute anything. Model preparation payloads contain only submitted material and relevant language choices, never learner recordings, application-store contents, secrets or repository .env files. Coach uses its approved local login for CLI work but accesses Phrase Gym data only through the API. Published practice works while Coach is offline as long as Phrase Gym is running.

## Backup and restore

Even for later personal production use, backup and restore are maintenance operations while the app is stopped. Stop Coach work, wait for pending browser saves, close Phrase Gym cleanly, then copy the entire data directory to a dated backup. This includes the database with pending preparation requests, reference audio, every saved take and settings. Never copy just a live SQLite main file while ignoring its journal.

Restore with the application stopped: keep the current directory as a fallback, copy back the whole backup and start the compatible application version. Check card counts, due history and representative reference/take playback. A manual documented procedure is enough for the first MVP; a later convenience command may automate these same steps.

No live restore, concurrent-backup protocol, lease invalidation, distributed snapshots, recovery service or high-availability machinery is required. A failed copy must not destroy the original. Backups are private learner data.

## Releases and data compatibility

Reuse useful Python code from Git and useful behavior/tests from both prototypes. Retain the Vue audio implementation and its real-device findings. Do not translate the Rust backend feature for feature or maintain two shipping backends. Change Vue and the new API together where that makes the reduced product simpler.

Prototype compatibility is not an MVP feature. Do not delete existing learner data or recordings during replacement. Use a clearly identified new application data directory; any later import of selected prototype content is an explicit, bounded action. Starting fresh does not authorize erasing old data.

Once the learner relies on the new application, use simple schema versions and upgrades while the app is stopped, preceded by a backup. Unknown schemas fail with an actionable message. Rollback uses the previous application version and its complete backup; it needs no concurrent operation support.
