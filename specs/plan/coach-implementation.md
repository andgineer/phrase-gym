# Coach implementation plan

The learner pastes a word or phrase, optionally with its source sentence, or pastes the sentence itself. Coach prepares the card automatically. Basic English cards, the Coach inbox, Gemini free recognition and the checked subscription cards work through the existing Coach page, API and practice screen, following the [designs selected for implementation](../decision-external-preparation-agent.md#preparation-by-provider). **Gemini subscription** still needs its own preparation design and qualification.

## 1. Route requests and handle service failures

Add **Coach provider** to Settings. **Autodetect** tries OpenAI subscription, Gemini subscription when qualified and available, Anthropic subscription, then Gemini free. A pinned subscription falls back directly to Gemini free. The pin stays saved for the next request. Implement explicit CLI overrides and bounded availability checks under the [routing decision](../architecture.md#coach-provider-selection). Use the existing global CLI environment and remove Coach-specific tool restrictions. Keep model and effort explicit on each call.

**Acceptance check** — Fake adapters demonstrate Autodetect order, pin persistence, subscription status or bounded probes, cache invalidation, free model replacement, quota exhaustion and verifier interruption. A service failure can move to the next permitted route under the same request ID. An input rejection ends the request. Exhausting all eligible routes produces one service failure in Coach. Recovery publishes at most once.

## 2. Qualify what learners will receive

Check changed prompts and validators on fresh held-out inputs with independent semantic review. Qualify the Gemini free recognition prompt; no real call has exercised it. Include bare words and short expressions where a single initial gives too little help, source errors, corrections and localized explanations. Keep [later language rollout](implementation-plan.md#5-extend-only-the-content-the-learner-next-needs) separate from checks using multilingual fixtures.

**Acceptance check** — Report materially wrong learner-facing results, false rejections, useful basic and enriched cards, time and usage by route and tested language. Qualification requires zero materially wrong results on that sample. Keep Gemini free on basic practice unless new evidence supports enrichment on natural drafts and planted errors, including misleading synonyms and changes of meaning. Verify the implementation through the ordered [repository gate](../../AGENTS.md#done-gate). Report content review, software checks and attended audio acceptance separately.
