# Small personal application

## Application boundary

Phrase Gym is one local Python application serving the retained Vue frontend and an HTTP API, with SQLite and local audio storage. The browser is the GUI. Use the normal browser on the learner's MacBook; no Qt, native Coach window, Electron bundle or additional GUI framework is needed. A launcher can open the browser later if starting the application proves inconvenient.

The Python server must work on both macOS and Linux. The MVP media player and recording experience target browsers on macOS; Linux browser audio support is deferred until after the MVP. Server portability does not expand the local-only access boundary.

The application owns cards, practice, scheduling, settings and recordings. Local reference synthesis, and later bounded AI preparation, run outside request handling in a simple background task. Already prepared practice never waits for a model or speech engine. Start with one preparation task at a time; there is no worker service, broker, remote inbox, lease protocol or multi-machine coordination.

Python is chosen for straightforward speech/model integration and iteration at personal-app load. Use ordinary libraries for HTTP validation, scheduling and speech rather than maintaining ports of their internals. Pin dependencies in a reproducible environment; an arbitrary system Python is not a packaging contract. Installer size and high-load performance are not product acceptance criteria.

## Responsibilities

Keep HTTP adapters, content validation/import, practice policy, persistence and preparation as focused modules in the same application. Vue renders state and owns browser audio effects; the server owns durable cards and schedule updates. Generate client API types from the Python contract. There is no need to preserve every prototype endpoint or module boundary.

Use one database and one application data directory outside the model's workspace. Keep original reference audio and learner takes with their card/attempt association. Ordinary SQLite transactions make a batch import and a rated review coherent. Retrying a completed import or review must not duplicate cards or count the same review twice. A single-writer application and stable operation IDs are sufficient; do not construct a general event platform.

Record the prompt, answer and support used for an attempt so later edits cannot misdescribe history. A replacement meaning starts a new card. Simple snapshots of those fields suffice; immutable profile registries, publication generations, cross-target dependency graphs and versioned job capabilities are not required.

## Growth after the MVP

Preserve stable card/source identities, attempt identities, recorded support and independent audio references. Keep scheduling policy separate from activity presentation and recording storage. Later [speaking activities](decision-transfer-and-engagement.md) can associate an attempt with several existing expressions without redefining a card or interpreting every recording as a memory review. Achievement and real-use records are separate from FSRS history.

Implement only the first MVP's records and APIs now. Add later activity records and optional content fields through ordinary schema migrations when their stages are built; no placeholder dialogue tables, generic workflow engine, plugin framework or speculative API endpoints are needed. Preserve existing MVP cards, grades and recordings through those upgrades. Version the import package format so later additions can coexist with the MVP format; unknown versions fail clearly, and compatible old packages remain importable.

## Local access and privacy

Listen on loopback and serve Vue and the API from the same origin. Retain basic Host/Origin checks against unrelated websites. There are no accounts, application credentials, synchronization or public endpoints in the MVP. The learner has authorized private iPhone access through Tailscale Serve. Keep the backend on loopback and opt in to one exact HTTPS machine origin; Tailscale controls which tailnet devices may reach the shared library. Do not use Funnel or a public listener. This adds private access, not per-user data separation or automatic iPhone audio qualification.

External course extraction happens outside the app. Imported source links are attribution, never instructions to fetch or execute anything. Model preparation receives only submitted material and relevant language choices, never learner recordings, database access, secrets or repository .env files. Published practice works without internet or an AI provider while the local application is running.

## Backup and restore

Even for later personal production use, backup and restore are maintenance operations while the app is stopped. Finish or cancel preparation, wait for pending browser saves, close the application cleanly, then copy the entire data directory to a dated backup. This includes the database, reference audio, every saved take and settings. Never copy just a live SQLite main file while ignoring its journal.

Restore with the application stopped: keep the current directory as a fallback, copy back the whole backup and start the compatible application version. Check card counts, due history and representative reference/take playback. A manual documented procedure is enough for the first MVP; a later convenience command may automate these same steps.

No live restore, concurrent-backup protocol, lease invalidation, distributed snapshots, recovery service or high-availability machinery is required. A failed copy must not destroy the original. Backups are private learner data.

## Releases and data compatibility

Reuse useful Python code from Git and useful behavior/tests from both prototypes. Retain the Vue audio implementation and its real-device findings. Do not translate the Rust backend feature for feature or maintain two shipping backends. Change Vue and the new API together where that makes the reduced product simpler.

Prototype compatibility is not an MVP feature. Do not delete existing learner data or recordings during replacement. Use a clearly identified new application data directory; any later import of selected prototype content is an explicit, bounded action. Starting fresh does not authorize erasing old data.

Once the learner relies on the new application, use simple schema versions and upgrades while the app is stopped, preceded by a backup. Unknown schemas fail with an actionable message. Rollback uses the previous application version and its complete backup; it needs no concurrent operation support.
