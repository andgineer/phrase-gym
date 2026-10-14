# P5 engineering review 3 — implementation corrections

The third and last authorized fresh review of `7e5eacc` identified three P2 findings accepted
by the coordinator. This batch fixes those findings without optional scope or provider calls.
There is no fourth independent review; the coordinator owns the convergence analysis and
confirmation handoff after these corrections.

1. Every configured referenced dialogue asset is confined, read, container/provenance validated
   and included in the source digest during dry-run. All unuploaded supplied media is reported;
   importable output refuses unresolved bindings. Upload retains actor/course originals and
   the worker uses supplied partner audio without synthesizing replacement.
2. Requested complete dialogue parsing no longer depends on audio-name selection. Each selected
   legacy note is parsed once for block/audio/mixed selectors; stable block suffixes are removed
   from phrase text while preserving attachment identity. Turn order, unmatched context and
   explicit ambiguity handling remain intact. Plain block notes and disabled automatic parsing
   with explicit dialogues remain supported.
3. Speech fallback catches the Edge SDK's `aiohttp.ClientError` transport hierarchy, including
   disconnects, HTTP errors and WebSocket handshakes. Same-locale fallback can complete; exhausted
   failure releases the lease and retains provider/audio checkpoints for explicit retry.
   Cancellation propagates and releases the worker lease. `aiohttp` is now a direct dependency;
   the lockfile was updated offline without changing any resolved package version.

Behavioral coverage includes missing/outside-root/symlink/corrupt supplied dialogue audio,
valid dry-run reporting and no-output rejection, byte/provenance preservation for course/actor
clips, worker reuse, block/audio equivalence, mixed selectors without duplicates, unmatched and
ambiguous turns, explicit dialogues, disconnect/HTTP/handshake fallback, exhausted partial-audio
recovery and cancellation. All media and provider boundaries are synthetic; no private learner
data or external source writes were used. The focused preparation suite passed 82 tests.

The final code gate ran in order and passed: `uv run inv pre` converged with every hook passing,
162 Python files unchanged and zero pyrefly errors; `uv run inv test` then passed 212 Python tests,
five client tests, actual frontend typecheck/build and 210 Chromium/WebKit browser checks.
Complete local outputs are `/private/tmp/phrase-gym-p5-review3-pre.log` and
`/private/tmp/phrase-gym-p5-review3-test.log`. The evidence-only closeout hook output is
`/private/tmp/phrase-gym-p5-review3-closeout-pre.log`.

Human content/listening, external sender rollout and actual-device/P6 acceptance remain open.
No independent post-correction review has established convergence after this final authorized
iteration. Implementation stops after the local correction commit for the coordinator's handoff.
