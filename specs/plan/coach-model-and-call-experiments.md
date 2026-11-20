# Coach model and call experiments

Choose the cheapest adequate model and effort for each CLI, test whether free Antigravity
can serve the learner, and make one-card preparation normally take one call, with at most
one justified repair. Five calls is a baseline to challenge, not a desired quality process.
This is a measurement plan, not evidence that any cheaper candidate already passes.
Use the [content decision](../decision-targeted-drilling.md),
[access boundary](../architecture.md#local-access-and-privacy) and
[standing authorization](implementation-plan.md#standing-authorization-for-the-local-english-demo).

## Explicit pins first

Every production, experiment, review and retry invocation must specify an exact model ID and
supported effort. Never inherit the interactive CLI selection or use moving aliases such as
`sonnet`, `opus`, `latest` or an unspecified effort. Record requested and reported model/effort;
mark identity unverified when the CLI does not report it. No silent model replacement.

1. Check rendered commands for all paths, including model rejection and developer harnesses.
   Claude currently specifies `claude-opus-5-5 --effort medium`; Codex specifies
   `gpt-6.1-sol -c model_reasoning_effort=medium`; Antigravity specifies
   `gemini-3.8-flash-high`, whose suffix is its effort.
2. Remove Antigravity's automatic selection of the newest listed Flash and its implicit
   medium-effort interpretation of suffix-free names. An unavailable pin must fail with a
   concrete reason. Model discovery may propose a replacement for a later explicit pin,
   but must not change a measurement or production run in progress.
3. Cover command rendering, missing effort, rejected pins and retries with focused tests.
   Keep the current Claude/Codex pins until comparisons qualify replacements. These are
   reference configurations, not a demonstrated cost optimum.

## Reuse news-recap's findings

Read the sibling repository's [agent research](../../../news-recap/spec/agents.md),
[cost evidence](../../../news-recap/spec/pipeline.md#cost),
[sandbox research and proposed probes](../../../news-recap/spec/plan-agent-sandboxing.md)
and launch pacing in `../news-recap/src/news_recap/config.py` before execution. Keep that
repository read-only. Its dated findings supply candidates and known limitations; they do
not qualify Coach cards or prove that every advertised model is available for free.

- The researched subscription-free path is Antigravity Gemini Flash, with explicit
  `gemini-3.8-flash-low`, `-medium` and `-high`. Flash-Lite is not exposed there. Do not
  spend time qualifying paid Claude models through Antigravity or rebuilding the old
  Gemini CLI route. Recheck the listed IDs before execution; a rename needs an explicit
  updated experiment manifest.
- Start with one Antigravity launch at a time and ten-second spacing, as news-recap does.
  Its measurements found roughly 12–13k input tokens of agent overhead per launch,
  six lost nights in 32 from quota lockouts, and five nights lost after a model rename.
  Free means no subscription payment, not unlimited throughput or negligible latency.
- Reuse file-based prompt delivery, JSON envelope/usage handling and existing error
  classification. The news-recap permission-bypass command is not Coach's access policy.
  Its native sandbox findings and proposed settings/probes are starting points, not
  evidence that all model tools are contained.

## Freeze a small, useful comparison

1. Reuse the seven inputs and intermediate outputs in the private
   `.phrase-gym-coach/coach-dev-both-20261116/` evidence. Freeze prompt/schema/validator
   hashes, CLI versions, exact commands, fixture IDs and budgets in a private manifest.
   The old commands did not pin both vendors; do not relabel those outputs as an
   equal-effort comparison of the current pins.
2. Add five synthetic development cases covering a short routine, a longer phrase,
   source preservation, a competing answer and a tricky gloss frame. Reserve eight fresh
   cases covering the same risks, including insufficient input, for confirmation only.
   No learner data, publication, Piper or source mutation is needed.
3. Set an explicit maximum launch count and existing per-call timeout before execution.
   Run two difficult development cases first for each candidate; stop candidates with
   clear material failures, auth/quota failures or unsupported settings. Run the remaining
   development cases only for survivors. Do not run a full model/effort/prompt/flow product.
4. Record every result, including failures: requested/reported model and effort, prompt
   bytes, tool turns, calls, elapsed time, input/output/thinking/cache tokens when exposed,
   deterministic findings and final disposition. Unknown usage stays unknown.
   Separate subscription quota, token usage, latency and dated API-price equivalents;
   API estimates are not subscription bills or a reliable quota conversion.

## Find which complexity earns its cost

Before new calls, inspect each saved draft, review and revision. The current flow is
prepare → review → conditional revise → review → conditional revise: two calls for a clean
card, up to five otherwise. Its final revision has deterministic validation but no further
semantic review. A new session of the same model is not an independent quality measurement.

1. Build a finding table with field, rule, before/after text, material learner harm,
   whether a validator could catch it, and whether the correction introduced another error.
   Count true corrections, cosmetic rewrites, false alarms and regressions per step.
   Explain each five-call case; do not infer value from the number of findings alone.
2. Audit the roughly 20 KB content rules against that table and the content decision.
   Classify each requirement as a learner-critical invariant, a mechanically checkable
   contract, optional help, or an unproven stylistic restriction. Keep source fidelity,
   correct sense, natural completed cues, usable partial hints and answer concealment.
   Challenge mandatory gloss variation, exact hint-letter fractions, companion uniqueness,
   forced multiple contexts and obligatory contrasts separately: identify the learner harm
   each prevents and the repairs or awkward wording it causes. A rule is not justified
   merely because a previous reviewer requested it.
3. Draft one short instruction set containing the critical invariants and a few examples;
   keep schema/count/length checks in deterministic validation rather than prose repetition.
   Compare it with the full prompt on the same pinned reference model and development
   cases, changing no other variable. Test a small rule group at a time only when findings
   identify a concrete problem. Record what can be deleted, moved to code or made optional.
4. Score correctness separately from richness. Two effective hints can be enough; a
   partner line or longer explanation earns its space only when it helps retrieve/use the
   target. Do not make Codex verbose to imitate Claude or count optional fields as quality.
   For each proposed simplification, inspect whether the actual study card remains useful.
5. Before executing changed prompts, specify expected improvements and semantic checks.
   Simplified rules remain experimental until a fresh review confirms the learner-critical
   bar. Update the authoritative content decision and prompt/validator rules together
   only after the measurements justify the change.

## Compare calls, then models

Use the reference pins to choose a shorter flow first. Keep the current flow as control;
reuse its archived intermediates where the prompts and pins actually match.

| Flow | Model calls | Question |
|---|---|---|
| Generate + deterministic checks | 1 | Is a separate model reviewer needed at all? |
| Generate + conditional combined check/repair | 1–2 | Can concrete validation failures or measured risk trigger the only follow-up? |
| Generate + always combined check/repair | 2 | Does checking every card prevent material defects missed by conditional review? |
| Existing separate review/repair rounds | 2–5 | What additional material defects does this cost prevent? |

1. A combined follow-up returns the corrected complete card or an unchanged card with a
   structured disposition, avoiding a review call that merely asks for another call.
   Apply deterministic checks after every response; reject a still-invalid card after the
   bounded repair. Evaluate semantic errors in unchanged cards and repair regressions too.
2. Define conditional triggers before the run, using observed risks such as explicit
   context ambiguity, not a model's uncalibrated claim that it is confident. If the trigger
   needs a model call to decide, count it. Insufficient input should finish in one call.
3. Count all launches, retries, failed attempts and discarded drafts. Report calls and
   latency per submitted item and per accepted card, plus median, maximum and acceptance
   rate. Do not hide rejected cards to make a cheap configuration appear efficient.
4. On the selected shorter flow, screen the cheapest researched candidates first:

   | CLI | Starting candidates from news-recap | Comparison |
   |---|---|---|
   | Claude | `claude-haiku-4-5-20251001`, then `claude-sonnet-5` | Explicit low effort if supported; medium on a promising candidate; reference `claude-opus-5-5` medium |
   | Codex | `gpt-5.6-luna`, then `gpt-5.6-terra` | Explicit low effort; medium on a promising candidate; reference `gpt-6.1-sol` medium |
   | Antigravity | `gemini-3.8-flash-low`, then `-medium`, then `-high` | Escalate only when the cheaper effort fails materially |

   These are dated research candidates, not claims of current availability or pricing.
   Verify access and supported effort without reading private settings/tokens; freeze the
   exact supported candidate before calling. If low effort is unsupported, record that and
   explicitly select medium only if supported; exclude models with no explicit effort control.
   Add a newer cheaper candidate only with availability/cost evidence. Test stronger effort on the same model before paying for the next model tier.
5. Keep the same fixtures, flow, prompt and reviewer rubric across models. Test the best
   cheap configuration against its reference on the reserved cases, with two runs per
   case to expose obvious variability. Stop after this bounded confirmation; investigate
   a failure with a specific hypothesis rather than reopening a general review loop.

## Fresh review and decision bar

A fresh reviewer, blind to provider/flow labels and separate from the generating sessions,
checks every returned comparison card and every claimed material correction. If an AI
reviewer is used, pin its model and effort and report its calls separately as measurement
cost, never as part of the proposed runtime pipeline. Inspect concrete results; schema
success and a model's approval of its own card cannot establish teachability.

Record material defects (wrong target/sense, source changes, misleading grammar/meaning,
answer disclosure, unusable hints), minor wording issues, optional richness and unjustified
rejections separately. Resolve contested findings against the learner task and authoritative
content decision, recording the concrete reasoning rather than demanding maximum detail.

A candidate qualifies on this finite set when all accepted confirmation cards satisfy the
critical invariants, insufficient inputs are rejected appropriately, and it has no more
material failures or unjustified rejections than the reference. A reference with material
accepted errors also needs correction; matching it is not enough. Report sample sizes and
residual uncertainty, not a universal quality or learner-effectiveness claim. Choose the
lowest observed quota/cost per acceptable card among passing configurations; if quota costs
are opaque, state that the cheapest candidate is not established and report the tradeoff.

## Antigravity verdict

Containment comes before card runs: Coach already observed a search tool reaching a home
file despite file-read denial. Reuse news-recap's native sandbox investigation and synthetic
read/search/command/write/network canaries. Check all exposed tool paths, deny access to
home, application data and `.env`, and verify headless completion and timeout cleanup.
Do not change global user settings, copy credentials, open login dialogs or build a new
container/worker architecture as part of model selection. If a per-run native configuration
cannot meet Coach's boundary, record containment as blocked and stop real card runs.

After containment passes, use the same low-to-high screen and confirmation cases. Repeat
only the winning free configuration's bounded confirmation on a second day to expose
obvious availability variation. Reuse ten-second pacing and stop on capacity/quota errors;
record service failures separately from bad cards. The prior draft/review followed by a
revision capacity error proves neither useful throughput nor that Flash is incapable.

Give separate verdicts for access, containment, card quality and practical throughput:
qualified for this sample, inadequate with concrete defects, unavailable/quota-limited, or
not evaluated because containment failed. Do not call a model plainly unusable because a
service was unavailable. If every completed candidate fails the learner-critical bar,
report the examples and conclude the tested free configurations are inadequate for Coach.

## Delivery

Publish a concise comparison with pins, sample sizes, semantic findings, acceptance rates,
calls/time/usage per accepted card, quota failures and the selected flow. Keep raw evidence
private under ignored `.phrase-gym-coach/`. Pin each qualified selection in code with
regression coverage, remove unjustified steps/rules, and update the authoritative specs.
Implementation uses the full repository done gate. Remove completed work from this plan;
retain only remaining experiments and lasting decisions in their focused specs.
