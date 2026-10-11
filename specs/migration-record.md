# Documentation consolidation — 2026-10-11

The user's final request was to place all project context under `~/projects/phrase-gym/specs/`, retain only the initial/legacy content in phrase-shadowing, and prepare for a new session. This operation changes documentation and organization only.

## Destination

- The 14 original specification documents are carried into `specs/`, with current status, directory names and the recording decision reflected in the relevant plans/contracts.
- The targeted-drilling decision and research record are preserved byte-for-byte.
- New documents record the start-here context, current session decisions, recording/comparison flow, concrete screen examples, agent guidance and the next-session brief.
- The original discussion export, pre-move source documents, legacy source snapshot, old visual explorations and stopped implementation are all retained under `specs/archive/`. See its [inventory](archive/README.md). They are not live instructions or accepted code.
- Root README and CLAUDE/AGENTS entry pointers lead a fresh session to `specs/START-HERE.md`. The existing user-documentation site stays under `docs/`.

## Old workspace

The active `spec/` and `discussion/` trees are removed from phrase-shadowing after verified destination copies and archives exist. Its README retains only the original trainer's lesson/audio/build documentation; agent guidance is reduced to that legacy scope. `index.html` and `phrases.py` remain unchanged, along with the existing AGENTS symlink and IDE files.

There is no dependency on the old workspace for understanding Phrase Gym's goal or design. Source-format documentation and a reference snapshot are included here. Actual lesson notes/audio in the vault and the separate echo-words project remain external source systems for later integration.

## What was not applied

The stopped implementation's code, fixture pack, CLI changes, dependency upgrades, tests and workflow modifications remain inside a labelled archive. They were not installed into the active app. `src/`, `tests/`, `pyproject.toml`, `uv.lock`, `tasks.py`, CI workflows and the original documentation-site scaffold retain their pre-move hashes. No new app prototype, package release, deployment, Git initialization, commit or push was performed.

## Verification

The move uses the [pre-move SHA-256 inventory](archive/migration-baseline-sha256.json), checks for concurrent edits and verifies the staged destination before removing old documentation. All active Markdown links/anchors, fenced JSON examples and ZIP integrity are checked. The accepted drilling/research documents, legacy trainer source and new app scaffold are checked against their original hashes. Code tests are not rerun for this documentation-only operation.

The next session should use [START-HERE](START-HERE.md) and the [next-session brief](plan/next-session.md), not the archived temporary status reports.
