# Coach model and call experiments

Choose the cheapest adequate model and effort for each CLI and determine whether free
Antigravity works for Coach. Start with one generation call and existing deterministic
validation. Add runtime review or repair only when measured learner-facing errors justify
it. Five launches is a baseline to challenge, not a required quality process.
Use the [content decision](../decision-targeted-drilling.md),
[CLI policy](../architecture.md#local-access-and-privacy) and
[standing authorization](implementation-plan.md#standing-authorization-for-the-local-english-demo).

## Explicit models and normal CLI environment

1. Specify a particular model ID and supported effort on every generation, review, repair
   and retry invocation. Never inherit the interactive model selection or use moving
   aliases such as `sonnet`, `opus` or `latest`. Reference configurations are
   `claude-opus-5-5 --effort medium`,
   `gpt-6.1-sol -c model_reasoning_effort=medium` and `gemini-3.8-flash-high`.
   Reference means a comparison point, not a demonstrated cost optimum.
2. Preserve automatic Antigravity fallback to another available free-tier model. Reuse
   news-recap's model discovery and lowest-effort Flash selection policy; pass the selected
   model and effort explicitly and log the switch. Do not silently infer effort from a
   suffix-free name: resolve a supported explicit setting. Never switch to paid models or
   another provider. Record requested and actual model/effort, keeping switched results
   separate in model comparisons. Mark reported identity unverified when not exposed.
3. Use the normal global CLI environment, existing login, settings, instructions, skills
   and tools as in news-recap. Remove Coach-specific tool restrictions and sandbox setup;
   do not add containment probes or make containment a prerequisite for card experiments.
   Preserve existing secret handling, bounded execution and private evidence storage.
   Do not change global user settings, copy tokens or create authentication workspaces.
4. Cover command rendering, explicit effort, free-only fallback and switch reporting with
   focused tests during implementation. These are planned runtime changes, not changes
   already made by this document.

## Reuse news-recap's findings

Read the sibling repository's [agent research](../../../news-recap/spec/agents.md),
[cost evidence](../../../news-recap/spec/pipeline.md#cost) and launch pacing in
`../news-recap/src/news_recap/config.py`. Keep that repository read-only. Reuse its CLI
execution, file-based prompt delivery, JSON envelope/usage handling, discovery, fallback
and error handling. Do not reopen its model-access research or build another framework.

- The researched subscription-free route is Antigravity Gemini Flash, with explicit
  `gemini-3.8-flash-low`, `-medium` and `-high`. Flash-Lite is not exposed there.
  Do not qualify paid Claude models through Antigravity or rebuild the old Gemini CLI
  route. Discover available free models before running; log any renamed replacement.
- Use one Antigravity launch at a time and ten-second spacing. News-recap measured
  roughly 12–13k input tokens of agent overhead per launch, six lost nights in 32 from
  quota lockouts, and five nights lost after a model rename. These are dated observations,
  not current quota guarantees. Free does not mean unlimited throughput.

## Inspect existing evidence before spending calls

1. Reuse the seven inputs and saved drafts/reviews/revisions in private
   `.phrase-gym-coach/coach-dev-both-20261116/`. The old commands did not pin both vendors;
   do not present these outputs as an equal-effort comparison of current configurations.
   The current flow takes two launches for a clean card and up to five with repairs;
   its final revision gets deterministic validation without a further semantic review.
2. Build a compact finding table: affected field/rule, before/after text, actual learner
   harm, whether code could catch it, and whether repair introduced a new error. Count
   true corrections, false alarms, cosmetic rewrites and regressions. Explain the
   five-launch cases rather than assuming every finding justified another call.
3. Audit the roughly 20 KB content rules. Preserve source fidelity, correct sense,
   natural completed cues, usable partial hints and answer concealment. Separate those
   invariants from optional richness and unproven authoring preferences. Challenge gloss
   variation, exact hint-letter fractions, companion uniqueness, forced multiple contexts
   and obligatory contrasts: what harm does each prevent, and what unnecessary repair or
   awkward wording does it cause? A prior review request is not sufficient justification.
4. Draft one short prompt containing the critical requirements and a few examples. Keep
   mechanical contract checks in code. Two effective hints can be enough; partner lines,
   more hints and longer explanations must earn their space. Do not make Codex imitate
   Claude's verbosity. Identify candidate rule removals before choosing stronger models.

## Run a small one-call screen

1. Select four existing inputs covering a phrase, a context-dependent word, a known
   gloss/hint failure and insufficient input. Reserve three fresh synthetic cases covering
   an ordinary phrase, a tricky word sense and insufficient input. Freeze prompt/schema/
   validator hashes, CLI versions, exact settings and fixture IDs in a private manifest.
   No publication, Piper, learner data or source mutation is needed.
2. Set a maximum launch count and existing per-call timeout before execution. Start each
   candidate on two of the development cases. Stop clearly failing candidates early;
   expand survivors to the four-case screen. Do not run a full model/effort/prompt/flow
   matrix. Quota/auth/capacity failures end the attempt without a retry loop.
3. Use one generation launch plus existing deterministic validation, with no mandatory
   model reviewer in the runtime flow. Inspect returned cards independently as measurement
   work. Start with the inexpensive candidates below; reference models are controls for
   specific failures or close choices, not a prerequisite full run for every configuration.

   | CLI | Candidates from news-recap | Effort comparison |
   |---|---|---|
   | Claude | `claude-haiku-4-5-20251001`, then `claude-sonnet-5` | Explicit low if supported; medium on a promising candidate; Opus 5.5 medium as reference |
   | Codex | `gpt-5.6-luna`, then `gpt-5.6-terra` | Low first; medium on a promising candidate; GPT-6.1 Sol medium as reference |
   | Antigravity | `gemini-3.8-flash-low`, then `-medium`, then `-high` | Escalate only for material failures |

   These are dated research candidates, not current availability or pricing claims.
   Check access and supported effort before calling. If low is unsupported, explicitly
   choose medium only if supported; exclude models without explicit effort control.
   Add a newer cheaper candidate only with availability/cost evidence. Test stronger
   effort on a promising model before paying for the next model tier.
4. For each failure, distinguish misleading content from an authoring preference or
   validator false alarm. If the latter drives the failure, test the proposed rule/prompt
   simplification on the same model and failing cases before escalating. Change one
   relevant variable at a time. Update the authoritative content decision, shared prompts
   and validators together when removing a requirement; never weaken tests to hide errors.

## Add a follow-up only if it earns its cost

1. If one-call cards pass the learner-critical bar, retain one call. If a concrete failure
   pattern remains, compare a single combined check/repair response with the original
   draft on those cases. It returns the corrected whole card or an unchanged disposition,
   avoiding separate review and repair launches. Validate every returned card and inspect
   regressions. After one repair, fail a still-invalid card with the concrete reason.
2. Prefer a conditional follow-up triggered by existing validation or a measured risk.
   Define triggers before testing. If selecting a trigger needs another model call, count
   it. Test unconditional checking only when evidence shows conditional checking misses
   material errors; do not pre-run every possible pipeline. Insufficient input should
   finish in one launch.
3. Confirm the cheapest passing configuration for each CLI on the three fresh cases.
   Repeat only a disputed or unstable result with a specific hypothesis; do not routinely
   double every case or add a second-day matrix. Expand only if concrete findings leave
   the selection unresolved. Preserve the reserved cases as confirmation, not prompt tuning.

## Measure actual work and judge the learner outcome

Record every concrete result, including failed, switched and rejected attempts. A CLI
launch can contain multiple model turns and tool calls. Track launches separately from
model turns/tool calls where exposed, prompt bytes, input/output/thinking/cache tokens,
elapsed time and deterministic/semantic findings. Unknown usage stays unknown. Count
retries and discarded drafts in work per submission and per acceptable card; report
acceptance rate and typical/worst observed latency and launches.

Separate subscription quota, tokens, elapsed time and dated API-price equivalents.
API estimates are not subscription bills or a reliable quota conversion. When quota costs
are opaque, report the tradeoff and call the winner the cheapest tested candidate supported
by the evidence, not a proven global optimum.

A fresh reviewer, blind to configuration labels and separate from generating sessions,
checks every returned comparison card and claimed material correction. If AI review is
used, specify its model/effort and report its calls as experiment overhead, not runtime
pipeline cost. Keep semantic review separate from deterministic conformance.

Score material defects (wrong sense/target, source changes, misleading grammar/meaning,
answer disclosure, unusable hints) separately from minor wording, optional richness and
unjustified rejection. Resolve disputed findings against the actual learner task. The
cheapest candidate qualifies on this finite sample when its accepted confirmation cards
meet the critical invariants, it rejects insufficient input appropriately and it does not
buy apparent quality by rejecting useful inputs. A flawed reference is not a passing bar.
Report sample size and uncertainty without claiming learner-effectiveness evidence.

## Antigravity verdict and delivery

Evaluate free Flash directly in the normal CLI environment using the same small screen.
Judge access, completed-card quality and practical throughput separately. A previous draft
and review followed by a revision capacity error proves neither usable throughput nor
model incapability. Capacity/quota errors mean unavailable or quota-limited, not bad cards.
If completed free configurations fail materially, report their concrete defects; if too
few runs complete, report quality as unresolved rather than plainly unusable.

Deliver a concise comparison with actual model/effort, fallback switches, sample sizes,
semantic findings, acceptance rates, launches/turns/time/usage and the chosen one-call or
bounded-repair flow. Store raw evidence under ignored `.phrase-gym-coach/`. Implement the
qualified settings and justified simplifications with focused regression coverage, then
use the full repository gate. Remove completed work from this plan; retain lasting
choices in the focused specs. No further broad research phase is required for handoff.
