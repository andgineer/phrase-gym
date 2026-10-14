# P5 engineering review 1 — implementation corrections

The independent review of `1e99e10` identified four P2 findings accepted by the coordinator.
This fix batch addresses exactly those findings; it makes no provider calls or optional expansion.
The coordinator owns the separate review-loop report and next fresh review dispatch.

1. Pending preparation now snapshots/fences all target, lesson and dialogue identities affected
   by publication. The versioned snapshot includes source provenance, absent/create cases, lesson title/language and
   ordered members with target revisions, and latest dialogue payload/revision/target bindings.
   Concurrent reviewed edits cannot be overwritten by a formerly completed draft. Older
   target-only snapshots are refused instead of silently rebound.
2. Generated and operator-corrected chunk labels are reconciled by exact required material across
   all retained target revisions/history: text, required status, frame-slot values and occurrence.
   Renamed/swapped labels map to verified unchanged material; changed or historically ambiguous
   identities receive distinct IDs. History is retained, never reset/deleted or relabelled.
   Normalized draft/package IDs and explicit mappings enter the review hash; publication refuses
   silent post-approval normalization. Old pinned manifests and late events keep their original
   identities, so they cannot contaminate changed material's current coverage.
3. Truncated/incomplete JSON fences raise a handled conformance error, retain the raw answer,
   release the failed lease, and recover through explicit regeneration. Malformed chunk/slot/
   candidate/uncovered references are validated before remapping in both worker and operator paths.
4. Read-only vault export rejects resolved runtime storage inside the source before creating any
   database/media/staging paths. This includes nested, symlink, vault-root-alias, case-alias and default config. Both runtime
   and export paths share the existing-ancestor inode check needed on case-insensitive filesystems.

Focused behavioral regressions cover stale existing/absent lessons/dialogues, membership and
member revisions, renamed/swapped/changed/ambiguous chunk IDs across older revisions, real-API
late pinned events, invalid draft references, repeated truncated-fence failures and regeneration,
and no-write vault runtime rejection. All provider/media fixtures are fake or synthetic; private
pack progress/audio and external source files remain unchanged.

The final code gate ran in order and passed: `uv run inv pre` converged with every hook passing,
156 Python files unchanged and zero pyrefly errors; `uv run inv test` then passed 182 Python tests,
five client tests, actual frontend typecheck/build and 210 Chromium/WebKit browser checks.
The 52 focused preparation regressions also passed. Complete local logs are
`/private/tmp/phrase-gym-p5-review1-pre.log` and `/private/tmp/phrase-gym-p5-review1-test.log`.
The evidence-only closeout hook log is `/private/tmp/phrase-gym-p5-review1-closeout-pre.log`.
All earlier human content/listening, external sender rollout and actual-device/P6 acceptance
gaps remain open; engineering review iteration 2 belongs to the coordinator.
