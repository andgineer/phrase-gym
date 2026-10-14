# P5 fresh-session engineering review loop — 2026-10-14

The user authorized P5 implementation in a fresh Sol 6.1 session followed by
fresh independent reviews, analysis and necessary fixes. The coordinator will
stop before a fourth review if actionable findings remain after three iterations,
explain the convergence problem and wait for confirmation. Reviewers start with
no conversation history and read the repository instructions and accepted decisions.
The separate real-provider semantic review is content evidence, not an engineering
review iteration or initial human/listening approval.

The implementation baseline is `1e99e10`, extending `b98414d`. Its ordered gate
passed all hooks with zero pyrefly errors, then 152 Python tests, five client tests
and 210 browser checks with actual typecheck/build and Chromium/WebKit execution.
See [implementation evidence](p5-implementation-2026-10-14.md) and the
[operator runbook](../p5-runbook.md). No push or deployment is authorized.

## Iteration 1

A fresh `gpt-6.1-sol` reviewer audited the complete P5 diff at `1e99e10` read-only.
It reviewed contracts, jobs, validation, publication, speech, source adapters,
CLI/API boundaries, tests, accepted decisions and content evidence. Focused
reproductions used disposable databases, fake providers and synthetic media.
No private data, real providers, full suites or browser servers were used.

Four P2 findings were independently reproduced and accepted by the coordinator:

1. A pending preparation snapshot includes only target fingerprints, although its
   package also replaces lessons and dialogues. Publishing a newer lesson title
   while preparation is pending, then publishing the prepared package, restores
   the older title. The correction must fence every affected lesson/dialogue
   binding as well as target content.
2. Generated chunk IDs can relabel target-level recall history. Swapping chunk
   IDs in an otherwise identical phrase attaches ten recalls for its beginning
   to its ending and misdirects cue selection. The correction must retain evidence
   only for verified unchanged required material, map renamed IDs explicitly and
   prevent changed material from reusing historical identities.
3. A truncated fenced provider answer such as ` ```json ` raises an uncaught
   `IndexError`. The raw response is retained but the job remains leased with no
   structured error and repeats the crash after recovery. The correction must
   produce a recoverable conformance failure and support explicit regeneration.
4. `export-vault` permits its application data directory inside the selected vault.
   Even a dry run then creates SQLite/media/staging files in the read-only source.
   The correction must reject resolved data/output paths inside the vault before
   opening runtime, including symlink aliases.

The reviewer found human content/listening approval, carried actual-device checks
and external echo-words sender source-write authorization to be valid explicit
dependencies rather than engineering findings.

The correction batch is committed at `2271507`; see the
[fix evidence](p5-review-1-fixes-2026-10-14.md). Package snapshots now fence target
and source provenance, lesson membership and current member revisions, and
dialogue revisions/bindings, including absent identities. Chunk reconciliation
compares all retained revisions by exact material, required status, slots and
occurrence; it records provider-to-stored mappings before human review hashes.
Changed or historically ambiguous material receives a distinct identity while
late pinned-session evidence remains attached to its original material.
Malformed fences become recoverable conformance failures. Vault data and output
paths are rejected before runtime writes, including symlink/default-directory
and actual Mac case aliases checked through filesystem identity.

Thirty added behavioral cases bring the focused P5 suite to 52 checks. The
ordered correction gate passed every hook with zero pyrefly errors, then
**182 Python tests, five client tests and 210 browser checks**, with actual
typecheck/build and both engines. Documentation-only closeout hooks also passed.
Iteration 2 follows in a new fresh session against the complete corrected tree.

## Iteration 2

A second fresh `gpt-6.1-sol` reviewer audited the complete P5 tree at `2271507`,
not only the correction commit. It found no additional concrete publication,
lease, confinement, chunk-history, pinned-session, reviewed-help/equivalent or
matching-language speech defect. Two new P2 findings were independently
reproduced with fake boundaries and disposable data, and accepted:

1. Speech preparation version participates in media identity but not job input
   identity or worker fencing. After reference synthesis succeeds and partner
   synthesis fails, a speech-version change deduplicates back to the older job.
   Retry then retains its checkpointed older reference, bypassing the correct
   media-cache invalidation. The correction must version job input and fence
   resumed work, including older payloads/checkpoints, while keeping published
   historical content readable.
2. Automatic vault dialogue linking selects the first text/attachment match.
   Selecting `Right.` with `right.mp3` under both agreement and direction meanings
   silently binds the dialogue to whichever selection appears first. The
   correction must reject ambiguous automatic bindings or require an explicit
   mapping; manifest order cannot choose the learning objective or schedule.

The correction batch is committed at `7e5eacc`; see the
[fix evidence](p5-review-2-fixes-2026-10-14.md). One versions module supplies
generation, review and speech revisions. Speech version participates in canonical
job input and worker checks; retained generated-audio checkpoints are versioned.
Legacy/mismatched queued payloads or unversioned audio fail explicitly before
provider/audio work, while historical completed jobs and published media remain
readable. Unchanged speech media remains reusable across unrelated text-model
or prompt changes. Automatic dialogue linking rejects multiple exact matches
independently of selection order; explicit complete bindings retain distinct
meanings, targets and schedules.

Thirteen added behavioral cases bring the focused P5 suite to 65 checks. The
ordered correction gate passed every hook with zero pyrefly errors, then
**195 Python tests, five client tests and 210 browser checks**, with actual
typecheck/build and both engines. Documentation-only closeout hooks also passed.
The independent finding count has narrowed **4 → 2**; none of the first round's
fixes was reported as recurring. Iteration 3 will use another fresh session.
A fourth review requires a convergence explanation and the user's confirmation.

## Iteration 3

A third fresh `gpt-6.1-sol` reviewer audited `b98414d..7e5eacc` across the full
implementation, importer/session behavior and behavioral tests. An interruption
was resumed in that same reviewer, rather than creating a fourth fresh session.
It returned three confirmed P2 findings, accepted by the coordinator:

1. Supplied dialogue assets bypass confinement, byte/container checks and upload
   reporting during dry-run. Missing/outside-root paths are accepted, and an
   importable CLI export can silently drop a supplied course partner clip. The
   worker then synthesizes replacement audio. Referenced dialogue assets must
   receive the same validation and transfer guarantees as phrase attachments.
2. Stable block-ID selection does not populate the parsed-note cache used by
   automatic dialogue export. `include_dialogues=True` silently omits a complete
   exchange that audio-name selection on the same note preserves. Dialogue
   inclusion must be independent of phrase selection mechanism.
3. Installed Edge speech propagates `aiohttp.ClientError` subclasses not caught
   at the speech boundary. A disconnect escapes matching-voice fallback and job
   failure handling, leaving the job leased with no error. Dependency network
   failures must follow the existing fallback/retry protocol.

Focused reproductions used disposable notes/databases, synthetic media and fake
SDK/provider boundaries. No full suites, provider calls or private-data reads
were performed. No other actionable finding was established.

The authorized correction batch is committed at `230b16d`; see the
[fix evidence](p5-review-3-fixes-2026-10-14.md). Referenced dialogue media is
confined, read, validated and hashed during dry-run, and all unuploaded supplied
media is reported and blocks importable export. Uploaded originals retain bytes
and actor/course provenance. Requested dialogue parsing is independent of the
phrase selector; mixed selectors preserve complete exchanges without duplication.
Edge client network exceptions follow matching-voice fallback and exhausted
retryable failure, retaining checkpoints; cancellation still propagates.

Seventeen added behavioral cases bring the focused P5 suite to 82 checks. The
ordered correction gate passed every hook with zero pyrefly errors, then
**212 Python tests, five client tests and 210 browser checks**, with actual
typecheck/build and both engines. Documentation-only closeout hooks also passed.
All nine confirmed engineering findings across the three reviews are corrected.
The third batch has not received an independent post-correction review, so
engineering convergence is **unverified**. The loop is stopped before any fourth
fresh review, pending the user's confirmation.

## Convergence analysis and proposed clarification

The independent finding counts are **4 → 2 → 3**. Previous fixes did not recur;
new reviews exposed different paths through source intake, media transfer and
provider recovery. The plan describes the intended results but does not give
one shared acceptance matrix for every supported path. Tests of a common path
therefore left another path's contract unverified. This is incomplete boundary
coverage, not evidence that the learning policy needs to change.

The following clarification is **proposed, awaiting user confirmation**:

- Apply the same source contract to block-ID and audio-name selectors: selected
  meaning remains explicit, requested complete dialogues survive, ambiguous
  bindings reject, and selector order never chooses a target or schedule.
- Apply one media contract to phrase and partner assets across dry-run, upload,
  importable export and worker resume: validate confinement/provenance/container,
  report supplied-but-unuploaded media, require durable bindings before export,
  preserve supplied originals, and synthesize only genuinely missing assets.
- Inventory actual locked provider/SDK network exceptions and verify fallback,
  exhausted failure, retained checkpoints, explicit retry, cancellation and
  version-upgrade behavior together. Handled provider failures must enter the
  retryable failed state; process crashes retain checkpoint/lease recovery.
- Trace these guarantees through the complete adapter → job → media → review →
  publication → session flow before any additional fresh review. Add omitted
  matrix cases, rather than restarting generic reviews with an unchanged plan.

After the third correction batch, request confirmation of this clarification and
one additional fresh Sol 6.1 review. Do not claim engineering convergence from
passing automated suites or the coordinator's own fix inspection. Human initial
content/listening approval and external sender source-write authorization remain
separate dependencies.
