# Production foundation and code boundaries

2026-10-12. P2 extends the original Python scaffold. The user confirmed `frontend/` as the client location and requested the echo-words coding rules, small focused modules and a single instruction source. `AGENTS.md` owns those adapted rules; `CLAUDE.md` is a symlink to it.

The current stack is Vue/TypeScript/Vite, FastAPI/Pydantic, SQLAlchemy/Alembic and SQLite. Source dependencies are locked. One local API process serves the built client and API from one origin. The CLI binds loopback; public/private HTTPS access and release authentication remain later work. No provider call or learner-audio upload is part of the foundation.

## Boundaries

| Area | Responsibility |
|---|---|
| `contracts/` | Shared validated content/settings/import/library/exposure/media shapes; no dependency on HTTP or domain services. |
| `api/` | Thin routers by endpoint family, request limits/origin boundary and service construction at startup. |
| `content/validation.py` | Structural identity, language, publication and media-binding validation. |
| `content/imports.py` | Staging, stale-content checks, explicit publication and transaction ownership. |
| `content/publication.py` | Target revisions, lesson membership and pinned dialogue revisions inside the caller's transaction. |
| `content/catalog.py` | Language-scoped read/navigation/search operations. |
| `settings.py`, `content/exposures.py` | Durable preferences and idempotent reference exposures, separate from recall scheduling. |
| `media/` | Original reference bytes, provenance, integrity, playback delivery and crash reconciliation. |
| `storage.py`, packaged `migrations/` | Database tables, migration execution and short serialized writes. |
| `frontend/src/components/` | Focused library, lesson, phrase-material and settings views. |
| `frontend/src/composables/` | Route orchestration, preference persistence and reference-audio effects. |
| Pure client reducer | Original-response/support snapshots; no authoritative schedule computation. |

Domain code uses contracts and persistence; it never imports routers or application startup. Services are composed once at startup and the CLI uses the same publication operations. Shared helpers have a direct owner; moved symbols have no compatibility/re-export shims.

Aim for at most 250 lines of handwritten production source per file and review before 300. Hooks enforce the 300-line ceiling and forbid domain-to-HTTP imports and local Python imports. Generated contracts and migrations are exempt from size limits. Client formatting and separate type checking keep small modules readable. Full tests are organized by domain responsibility and include real-browser behavior.

Content revisions hold their initial realization/chunks/cues as immutable JSON. Targets use internal UUIDs; stable source references do not depend on array position or wording similarity. Multiple lesson/dialogue appearances share a target. Explicitly changed objectives require a new source identity; cross-source reconciliation remains future intake work. Dialogue revisions pin target content revisions so later editorial changes do not rewrite old material.

Media keeps original bytes, with metadata identity incorporating content hash and provenance/preparation settings. Manual publication checks exact language/text/role. Missing media is an explicit preparation gap. The initial search is Unicode substring matching over text, purpose and prepared concepts; it is intentionally independent of lesson filtering.

These boundaries support P3 sessions, recording retention and scheduling without turning the importing service into a catch-all. P3 must keep session planning, evidence processing, review policy and recording transfer separate, using the accepted unified-card interaction. P4/P5/P6 retain their device/offline, provider-intake and deployment responsibilities.
