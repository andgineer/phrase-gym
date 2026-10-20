# Rust preview access

The Rust preview requires explicit scoped credentials. Keep their JSON file outside Git with owner-only permissions. Generate distinct random URL-safe tokens (at least 32 characters); never reuse the examples or put real tokens on the command line. Each array entry has an `actor`, `token` and `capabilities` list:

```json
[
  {"actor":"learner","token":"REPLACE_WITH_A_RANDOM_LEARNER_TOKEN","capabilities":["learner"]},
  {"actor":"sender","token":"REPLACE_WITH_A_RANDOM_SUBMISSION_TOKEN","capabilities":["material_submit"]},
  {"actor":"coach","token":"REPLACE_WITH_A_RANDOM_WORKER_TOKEN","capabilities":["worker"]},
  {"actor":"preferences-agent","token":"REPLACE_WITH_A_RANDOM_PROFILE_TOKEN","capabilities":["profile_read","profile_update"]}
]
```

Build the retained client, then launch with a disposable data directory:

```sh
npm --prefix frontend run build
cargo xtask run --credentials-file /private/path/credentials.json --data-dir /private/tmp/phrase-gym-preview
```

The equivalent binary command is `phrase-gym-server serve --credentials-file /private/path/credentials.json --data-dir /private/tmp/phrase-gym-preview --client-dir frontend/dist --port 8010`. Open `http://127.0.0.1:8010/login` and enter the learner token. Only a learner-only credential can establish the HttpOnly, SameSite=Strict browser cookie. The cookie is session-scoped; sign in again after closing the browser. API clients send their credential in the `Authorization: Bearer …` header. Cookies never authorize preparation or profile administration. Rotate a credential by updating the private file and restarting Server; existing cookies with the old token cease to authorize access.

Server binds loopback. Remote operation requires the separately authorized private HTTPS boundary described in [architecture](../specs/architecture.md#private-release-and-recovery); host/origin allowlists alone provide no authentication. The login cookie is Secure when initiated from HTTPS. This document does not establish deployment, recovery, real CLI, content or actual-device acceptance.

The initial personal profile records the [specified learner preferences](../specs/decision-personal-preferences.md). `GET /api/v1/profile` and `GET /api/v1/profile/revisions/{revision}` require profile-read, worker or learner authority. `PUT /api/v1/profile` and `POST /api/v1/profile/restore` require profile-update authority. Mutations include their unique mutation ID, expected current revision and reason. A restore references an existing revision and appends a new revision. Identical authorized retries recover the original revision even after later edits; conflicting payloads, actors or stale expected revisions fail without changing the profile. Device settings remain independent. Worker access to pinned generation inputs and publication uses the durable English inbox protocol.

The generated Rust contract includes `PreparedEnglishLesson` for one lesson of one to five short fixed English phrases. It pins a personal-profile revision plus guide and assessment versions, groups stable source identities, supplies one or two situations, optional partner turns, one partial hint and original reference media. The core validator checks structure, identity/revision preconditions, supplied-audio integrity and exact role/text/language bindings. It rejects gapped fields, complete/accepted answers in prompts or lesson titles, unresolved English preferences and unsupported kinds. These checks do not certify semantic prompt quality or language usage. The production English inbox and fenced atomic publication path is implemented. These commands establish Server access. Remaining Coach integration and generated-lesson acceptance are tracked in the [implementation plan](../specs/plan/implementation-plan.md#m2-generated-lesson-demo-on-local-macos).
