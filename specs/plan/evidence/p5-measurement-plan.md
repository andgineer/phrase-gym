# P5 measurement plan — fixed before provider calls

Date: 2026-10-14. Public authored fixtures only; no learner recordings or private vault text.
Development and held-out fixture membership is fixed in `p5-fixtures/sources.json` before calls.

The development set covers the broad day-response cue failure, reciprocated greeting with
beginning/ending masks, idiomatic selected sense, word-target leak, one-variant short phrase,
and Serbian Latin script. The held-out set adds a long construction, a noninitial distinguishing
anchor with competing phrases, contractions, German idiom, Serbian Cyrillic, and a missing-context
single-word expression where no reliable partial cue is expected. Native help is Russian.

Use the explicitly configured llmbroker free pool, one complete answer at a time, no paid
escalation. Inspect key/model availability first. Budget is 24 text calls (12 generation +
12 advisory review), at most 60 seconds each, and at most one full-phrase speech generation
per fixture. No runtime cue generation or learner-effectiveness measurement. Do not tune on
the held-out results; any later prompt tuning requires a newly held-out set.

Record every raw answer, exact input, model/provider, prompt version, latency, token usage,
conformance failure, advisory review and media identity. Use only the configured free pool; record any provider-reported monetary cost. Token usage is
not a monetary invoice and no account invoice is inspected by this harness. Failed speech/drafts remain withheld.
Fresh semantic review inspects every concrete answer, separating candidate ambiguity/meaning,
help/search and schema conformance. A fresh agent is a semantic reviewer, not an operator
listening attestation. Actual human initial content review and reference listening approval
remain required before generated publication. Software fixtures use synthetic clips and
explicitly labelled fake approvals; they cannot establish speech quality.

## Prompt revision 2 — development failures and a replacement held-out set

The first pool run failed 8/12 conformance checks: seven empty visible-chunk situation cues,
and one malformed JSON answer. Four reached withheld drafts with real generated Edge audio.
Revision 2 is based on development failures: explicitly prohibit empty visible chunks, clarify
that fixed phrases hide a subset of meaningful chunks, include the accepted reciprocated-greeting
mask example, avoid punctuation-only chunks, and default equivalents to empty. Original raw
held-out failures remain evidence, not an independent validation set for revision 2.

Before revision-2 calls, `sources-v2.json` fixes six development cases and six NEW held-out cases:
call-it-a-day idiom, long meeting confirmation, shouldn't-have frame, German anticipation frame,
Serbian station question, and missing-context `Fine.`. Budget: another maximum 24 free-pool text
calls and 12 speech jobs. Do not tune against this new held-out set within this run.
