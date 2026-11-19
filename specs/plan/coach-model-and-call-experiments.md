# Coach model and call experiments

Choose the lowest-cost Claude and Codex configurations that produce teachable cards, determine
whether the free Antigravity tier is usable, and reduce calls per card. Keep the current pins and
preparation flow while measuring candidates. Use the [content rules](../../src/phrase_gym/coach/prompts/content-rules.md),
the [local access boundary](../architecture.md#local-access-and-privacy), and the
[standing demo authorization](implementation-plan.md#standing-authorization-for-the-local-english-demo).

## Baseline and fixtures

Freeze the prompt versions, schemas, current pinned adapters and the current preparation and
review flow. Use the same examples for every comparison. Include a phrase alone, a phrase with
a source sentence, a polysemous word with a source sentence, a confusable word, and input with
no teachable target. Include cases where previous reviews caught a wrong sense, an answer-bearing
cue or a gloss that did not fit the sentence. Keep a separate set of fresh examples for the
final comparison. Use synthetic material and keep raw results under the ignored
`.phrase-gym-coach/` directory.

Before running each configuration, check that the model is available under the intended login.
Record exact model and effort, result or failure at each step, elapsed time, total calls and any
usage the CLI reports. Track availability separately from content quality. A fresh reviewer
checks every returned card against the source and content rules, without seeing the model or
pipeline label. Require correct target and sense, usable answer-free cues and hints, natural
English, and the expected rejection of insufficient input. Record semantic findings separately
from schema and content-validator results.

## Cheapest adequate pins

1. Run the frozen examples through Opus 5.5 medium and GPT-6.1 Sol medium to establish the
   baseline at the actual pins. Earlier Claude results at a different effort are context only.
2. Use each CLI's current model list and available usage information to choose a cheaper
   candidate, starting with Claude Sonnet and a lower-cost Codex model if available. Compare
   lower effort separately so the effect of model and effort can be distinguished. If a candidate
   fails, compare the next affordable configuration rather than changing card rules to pass it.
3. Confirm the cheapest candidate that passes on the fresh examples before changing its pin.
   Where quota units are unavailable, report calls, time and any token counts as separate
   measures. Subscription quota and API price estimates are different measures.

## Fewer calls per card

1. Save the card after each existing step. Compare the initial, reviewed and revised versions
   to see which changes corrected a material defect. Review the final revision as well, since
   revision can introduce a new error.
2. Start with preparation plus deterministic validation. Test a single follow-up that checks
   and corrects the card in one structured response. Compare that with a conditional follow-up
   for validation failures and for concrete risks identified by the baseline, such as a
   context-dependent sense. Measure missed defects as well as saved calls.
3. Compare the best shorter flow with the full baseline on the fresh examples using the
   same models and validators. Keep a step only if it reliably fixes material defects the
   shorter flow misses. Limit repairs explicitly and fail with the concrete reason if the
   final card still fails validation. Report ready-card rate, typical and worst-case calls,
   latency, and semantic defects together.

## Antigravity verdict

Confirm access with a Google login on the free tier. Use the highest-effort available Gemini Flash
and the same fixtures and review standards. Report successful cards, timeouts, service capacity
errors and exhausted quota separately. The initial phrase run returned a draft and a review, then
failed revision with a service-capacity error. It did not establish usable card quality or
throughput.

Check containment first with synthetic files and no learner data. Verify that every tool exposed
to the model is mechanically denied access outside the attempt folder to home, application data
and `.env` files. A file-read denial alone is insufficient. Use only synthetic probes until this
passes. If required containment is unavailable, or availability or card quality is insufficient,
record the observed limitation and keep the option experimental and unrecommended.
