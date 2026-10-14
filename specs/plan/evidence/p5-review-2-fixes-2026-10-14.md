# P5 engineering review 2 — implementation corrections

The fresh independent review of `2271507` identified two P2 findings accepted by the coordinator.
This focused batch addresses those findings without optional expansion or provider calls.

1. Shared preparation revisions now make speech version part of canonical durable job input.
   Workers refuse missing/mismatched queued speech versions before any external call and stamp
   generated-audio checkpoints. Unversioned/mismatched retained audio is withheld; explicit
   regeneration archives it instead of silently resuming. New-version enqueues have distinct
   identities while historical completed jobs, human hashes and published content stay retained.
   Speech cache identity remains independent of unrelated generation/review/provider changes.
2. Parsed vault dialogue turns no longer choose the first matching selected meaning. Multiple
   exact text/attachment matches reject export with an order-independent ambiguity report;
   zero matches remain honest partner context and unique matches retain exact linking.
   Existing explicit complete dialogue mappings remain usable with automatic parsing disabled.

Behavioral regressions cover same-version partial resume, version-changed partial refusal,
failed/ready deduplication, legacy input/checkpoint handling, explicit regeneration, historical
publication preservation, unrelated LLM changes reusing identical speech media, dialogue order
independence, zero/unique matches, and explicit meanings with distinct target IDs/schedules.
All provider/media boundaries are fake or synthetic; external sources and private learner data
remain untouched. The focused preparation suite passed 65 tests.

The final code gate ran in order and passed: `uv run inv pre` converged with every hook passing,
159 Python files unchanged and zero pyrefly errors; `uv run inv test` then passed 195 Python tests,
five client tests, actual frontend typecheck/build and 210 Chromium/WebKit browser checks.
Complete local outputs are `/private/tmp/phrase-gym-p5-review2-pre.log` and
`/private/tmp/phrase-gym-p5-review2-test.log`. The evidence-only closeout hook output is
`/private/tmp/phrase-gym-p5-review2-closeout-pre.log`.

Human content/listening approval, external sender rollout and actual-device/P6 acceptance remain
open. The coordinator owns the separate review-loop record and last authorized fresh review.
