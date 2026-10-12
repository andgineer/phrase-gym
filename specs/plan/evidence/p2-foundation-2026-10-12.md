# P2 foundation evidence — 2026-10-12

The user requested beginning P2 and committing directly to main, confirmed `frontend/` for the Vue/TypeScript client, then requested the echo-words coding rules and active supervision of architecture/file structure. This record concerns the production foundation; P3 has not started and real-device/content acceptance remains open.

## Implemented scope

- FastAPI/Pydantic API and same-origin Vue client, explicit data/client directories, loopback runtime and packaged Alembic migrations.
- Persistent native/learning UI settings and separate native explanations; language-scoped bookmarks, invalid/empty states, reload/history and independent tabs.
- Validated, staged, explicitly published manual packages for fixed phrases, word senses and frames. Stable source-to-target identities, immutable revisions, shared lesson memberships, pinned dialogue targets/revisions and atomic stale-content rejection.
- Lesson-title filtering, phrase/concept search, phrase materials and complete lesson lists. Idempotent reference exposures without recall grades, with a small acknowledgement-based client outbox.
- Original reference-media bytes, immutable hash/provenance identities, exact text/language/role checks, gesture playback, byte ranges, missing/corrupt recovery and conservative crash reconciliation.
- Shared contracts below HTTP; focused importing, validation, publication, catalog, settings and exposure services; separate API routers and Vue views/composables. No catch-all content service or re-export shim remains.
- `AGENTS.md` holds adapted coding rules; `CLAUDE.md` is a symlink to it. Hooks enforce domain/transport direction, top-level imports, a 300-line handwritten source ceiling and client formatting. The largest backend domain module is 189 lines; the route composable is 239.
- Locked dependencies, OpenAPI snapshot/generated client types, frontend CI and a build script that bundles the client into Python distributions. Existing scaffold tooling is retained.

## Observed verification

`uv run inv pre` completed successfully, followed by `uv run inv test`:

| Check | Result |
|---|---|
| Repository hooks | YAML, hygiene, Ruff, Python formatting, pyrefly, architecture guard and client formatting passed; pyrefly reported 0 errors. |
| Python suite | **33 passed**. Covers migration/schema parity, restart settings, publication/retry/reimport, incompatible objectives, structural rejection, stale staging, whole-transaction rollback, concurrent publication, shared identities, historical dialogue bindings, language scopes, lookup and exposure receipts. |
| Media/API cases | Included in the Python suite: original bytes/provenance, idempotent upload, byte ranges, corrupted/truncated clips, missing-file recovery, incorrect language/role/text, answer-leaking cue rejection, orphan retention, request-size/host/origin boundaries. |
| Client type checking | Passed independently of the Vite build. |
| Pure reducer | **4 passed**: original-response snapshot, support from prior materials, no overwrite by later takes/help, and invalid language handling. This is evidence-state groundwork, not an implemented session engine. |
| Real-browser walkthroughs | **14 passed**: seven scenarios in each of Chromium and WebKit. First launch/reload, lookup → explanation → full lesson → Back, tabs/bookmarks/history, native UI/empty language, gesture playback, failed exposure acknowledgement/reload recovery, narrow viewport/200% text and safe rendering of imported HTML-like text. |
| Visual inspection | The narrow/enlarged-text materials screenshot was inspected; long imported text wraps and is rendered literally, with no horizontal overflow. |
| Documented build | `./build.sh` completed: locked dependency installation, independent client types/build and hatchling wheel/source distribution. |
| Isolated wheel | With the locked runtime set and Python 3.13, the installed wheel served its bundled client, migrated an empty SQLite database and staged/published/searched a fixture package. Migrations/client are present; no `.env` file is bundled. |
| Dependency audit | `npm audit` reported **0 vulnerabilities** after selecting patched Vue/Vite/Vitest versions. |

The first combined build/test attempt overlapped dependency installation and interrupted Vitest. The full gate above was rerun sequentially after the build; that earlier partial run is not the accepted verification. Pre-commit environment setup initially hit the approval reviewer's account usage limit; after the user's intervention, the required hooks ran successfully. Neither is an unresolved implementation check.

Python checks ran locally on Python 3.13.13; client checks used the available Node 24 LTS runtime. The committed CI matrix includes Python 3.12/3.13 and client Node 22. Those remote matrix runs have not been observed because this work is local and unpushed. Existing Click and Starlette test-client deprecation warnings do not fail the suite.

## Remaining work and limits

- P2 is underway, not a release acceptance claim. Real actor/course audio and reviewed Serbian content/whole-phrase listening checks remain incomplete. Browser audio fixtures are labelled synthetic tones. MP3/Ogg/M4A validation inspects container/duration rather than fully decoding speech.
- Actual Safari/M1/external-headset clipping, iPhone Safari/installed-PWA capture, interruptions and persistence remain unverified. These tests exercise reference playback and navigation; P2 does not implement microphone capture or prove the reported problem resolved.
- Sessions, due scheduling/FSRS, learner capture, every-take SQLite BLOB retention and the IndexedDB pending-audio journal are P3. Start session and lesson-drill controls remain disabled until those services exist. The selected-lesson due-only/all policy still needs resolution before that session branch.
- The initial content revision has one realization and reviewed cue variants. Cross-source reconciliation and richer intake come later; no text-similarity merge or provider-generated publication is present.
- Search uses prepared concepts and Unicode substring matching. Real retrieval quality and the final content inventory are not established by four structural example targets.
- Offline service workers, private HTTPS access/authentication, verified backups, deployment sizing and provider/source integration remain later phases. No echo-words/vault/Anki files were modified, provider calls made, user recordings uploaded, push performed or deployment initiated.

Continue from the [handoff](../next-session.md) and [runbook](../p2-runbook.md), preserving the [unified-card decision](../../decision-unified-practice-card.md).
