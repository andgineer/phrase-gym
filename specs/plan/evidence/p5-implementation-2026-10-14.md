# P5 implementation and measurement evidence — 2026-10-14

The user authorized all P5 and a separate fresh engineering review/fix loop. This supersedes
historical no-P5/no-review scope restrictions. The implementation extends the Python scaffold
and Vue client contract on main; no source repository, vault note, Anki data, deployment or
private `.data/vault-test` progress/recording was changed. No push occurred.

## Delivered engineering boundaries

- Explicit `SourcePackage`, prepared candidate/draft/advisory/human-review, vault and handoff
  contracts. The source owns text/selected intent; the generator cannot replace objective IDs.
- SQLite leased jobs with input/settings/prompt identities, deduplication, three bounded attempts,
  token/expiration fencing, renewed checkpoints, restart recovery, retained raw answers,
  malformed-output regeneration history and operator draft revisions. Configuration/prompt
  mismatches are explicit failures; providers run outside database transactions.
- Versioned generation/adversarial-review prompts, reconstruction/target/slot/visible-anchor/
  comparable-mask checks, one-variant and explicit no-reliable-cue supported-study paths.
  Candidate situations and complete source dialogue exchanges remain ordinary fixed practice.
- Hash-bound initial human approval and atomic importer/job publication. Existing target IDs,
  schedules, recordings, earlier revisions/session manifests and other native-language help
  survive regeneration. A concurrent source revision refuses stale publication.
- Whole-phrase Piper and Edge speech with verified native model/config bytes, correct locale/
  accent fallback restrictions, Serbian speech-only Cyrillic conversion, exact text/input/voice/
  settings/version provenance and local cache. Supplied actor/course assets are preferred.
  Speech failure is retryable; practice never depends on live generation.
- Read-only selected-note parser for the actual legacy front matter, numbered `## фразы`,
  Obsidian attachments/situations, and `## диалоги` complete speaker turns. External stable IDs,
  exact attachment links, missing/ambiguous/duplicate reports and root/symlink confinement.
- Portable selected-sense envelope, same-transaction durable receiver/enqueue, restart/idempotent
  acknowledgement and changed-sense/content conflicts. Exact uploaded media IDs are supported;
  arbitrary URL fetching is absent. Manual demonstration is executable independently of UI.
- CLI preparation/configuration/inspection/retry/correction/publication, source export/regeneration,
  API intake/status/retry/handoff, generated OpenAPI/client types, and a real built-client lifecycle
  check covering generated stored masks, later-session rotation, concept search and pinned history.

The [runbook](../p5-runbook.md) gives executable commands and explains the external sender boundary.
An actual selected existing vault lesson note/clip dry-run passed (one selected phrase, exact audio
container/provenance checks); before/after source hashes matched. The manual envelope was received
by two separate CLI processes with the same durable job ID, proving receiver restart identity.

Production handwritten modules remain below the architecture limit and separate intake, lease,
provider/speech, deterministic validation, publication and source delivery responsibilities.

## Real provider and audio measurements

The [plan](p5-measurement-plan.md) fixed public development and held-out fixtures before budget.
The first measured prompt had eight failed jobs and four withheld drafts. Development failures
motivated revision 2; a new held-out set was fixed before its calls. Revision 2 has four failed
jobs and eight withheld drafts. All 24 generation answers and 13 advisory answers are retained,
including malformed output, empty visible-chunk failures, incomparable masks and a German review
that omitted its required `source_id`. The free pool had four usable providers/five configured
models at inspection; all 37 concrete answers came from `google-gemini-3.5-flash-lite`.

Concrete answer latency: median **1.626 seconds**, range **0.771–2.681 seconds**; **73,771** reported
tokens. One additional pool request timed out without an answer and a crashed worker lease was
recovered. Request/lifecycle exception handling and typed optional-setting normalization were
corrected following that concrete failure. No paid escalation or paid comparison was used;
the provider returned no monetary cost and no account invoice was inspected. Monetary cost is
therefore unverified rather than inferred from token counts; routing was explicitly through
the configured free pool.
See the [machine-readable summary](p5-measurement-summary.json), [original results](p5-results/)
and [revision-2 results](p5-results-v2/).

A fresh independent Sol 6.1 reviewer read every concrete result and every candidate, separately
from automated conformance: [semantic report](p5-semantic-review.md). Across 29 candidates,
14 have definite submitted defects and 15 require initial human judgment. This does not demand
mathematical uniqueness of every possible paraphrase or reopen the accepted illustrative masks.
It identifies actual context/help/equivalent/mask defects and advisory-review mistakes.
**No real generated target was published or operator-approved.**

Real speech was generated for 12 full-phrase Edge jobs and two native Piper samples. The retained
public EN/DE/SR Edge MP3s and native EN/DE WAVs are in [audio evidence](p5-audio/), with exact metadata,
hashes and positive playable-container checks. Native repeat requests returned the same asset IDs.
No actual human listening, pronunciation/accent acceptance or device/headset approval was performed.
Those samples are review artifacts, not a certified language pack or learner efficacy experiment.

## Software verification

Focused P5 Python boundary suite covers source edit/identity reports and complete existing
dialogue export in addition to the preparation boundary checks. The new lifecycle
check passed against the real server/built client in both Chromium and WebKit. Tests use synthetic
media and clearly labelled test approvals; no automated test calls a real provider, reads private
keys or learner recordings, or delivers to an external source.

Final ordered code gate passed: `uv run inv pre` (session 68612; every hook passed,
149 handwritten/source Python files unchanged, pyrefly zero errors), then `uv run inv test`
(session 35169): **152 Python tests, 5 client tests, 210 browser checks**, with actual frontend
typecheck/build and Chromium/WebKit execution. No frontend suite skipped. The complete full-suite
log is `/private/tmp/phrase-gym-p5-final-test.log`. Earlier 150-test evidence predates the final
source-report/dialogue-export scope corrections and is not the final gate.

Documentation/evidence changes after that code gate receive a further captured all-pass preflight
at `/private/tmp/phrase-gym-p5-closeout-pre.log`; no production/test change follows the ordered gate.
The local main commit and subsequent engineering review evidence are reported by the coordinator.
Software results establish engineering behavior only; acceptance dependencies below remain open.

## Remaining acceptance and authorization dependencies

- Initial operator human content review, corrections and actual listening approval of the measured
  pack are required. Real candidates are withheld. Missing-context meaning, wrong-language help,
  target leaks and prompt/mask mismatch cannot be silently published from model approval.
- The private 44-phrase pack has an executable regeneration path; it was preserved and not newly
  generated/published in this session. Initial cue/content/audio review is still required there.
- Real actor/course and Serbian content acceptance remains incomplete. The adapter can preserve
  existing provenance/assets; a source export is not semantic or listening certification.
- Echo-words' actual selection-time outbox, destination UI/action and unavailable-receiver retry
  require separate authorization to edit that external repository. The Phrase Gym receiver and
  portable manual envelope are implemented and demonstrated; existing Anki output was not switched.
- Existing P4 iPhone, wider Safari/headset and P6 private deployment/backup/restore acceptance
  remain separate. Positive 2026-10-13 Safari/M1 recording evidence stays valid for its tested setup.
