# Coach implementation plan

The learner pastes a word or phrase, optionally with its source sentence, or pastes the sentence itself. Coach prepares the card automatically. Basic English cards, the Coach inbox and Gemini free recognition work through the existing Coach page, API and practice screen. The subscriptions still need their checked enrichment:

- On an **OpenAI subscription**, `gpt-5.6-luna` writes examples and cues. In a separate call, `gpt-5.6-terra` checks the input and every addition. Both use low effort.
- On an **Anthropic subscription**, `claude-haiku-5-5` writes and `claude-sonnet-5` checks in a separate call, both at low effort. On either subscription, keep useful verified examples or give the learner a basic card for the confirmed target.
- **Gemini subscription** still needs its own preparation design and qualification.

These are the [designs selected for implementation](../decision-external-preparation-agent.md#preparation-by-provider).

## 1. Publish checked examples on subscriptions

Connect both writer/checker pairs. Check the proposed decision against the original, including its language, target and any correction. Check every complete example, gloss and hint. Render instructions, help and inbox reasons in their configured languages. Let accurate short translations such as Russian **яд** pass validation without padding.

**Acceptance check** — A verified example reaches the practice card with a gap, meaning cue and matching reference for the revealed wording. If only an addition is wrong, drop it and retain basic practice for the confirmed target as needed. If the calls disagree on the target, show the unresolved result in Coach. A verifier timeout leaves the draft unpublished. Exercise each outcome through the API, and cover basic and enriched practice in the real browser.

## 2. Route requests and handle service failures

Add **Coach provider** to Settings. **Autodetect** tries OpenAI subscription, Gemini subscription when qualified and available, Anthropic subscription, then Gemini free. A pinned subscription falls back directly to Gemini free. The pin stays saved for the next request. Implement explicit CLI overrides and bounded availability checks under the [routing decision](../architecture.md#coach-provider-selection). Use the existing global CLI environment and remove Coach-specific tool restrictions. Keep model and effort explicit on each call. Use news-recap’s existing free Flash discovery and pacing.

**Acceptance check** — Fake adapters demonstrate Autodetect order, pin persistence, subscription status or bounded probes, cache invalidation, free model replacement, quota exhaustion and verifier interruption. A service failure can move to the next permitted route under the same request ID. An input rejection ends the request. Exhausting all eligible routes produces one service failure in Coach. Recovery publishes at most once.

## 3. Qualify what learners will receive

Check changed prompts and validators on fresh held-out inputs with independent semantic review. This includes the Gemini free recognition prompt, which now reads the learner's single typed text and answers in the response text of news-recap's Antigravity command; no real call has exercised it yet. Include bare words and short expressions where a single initial gives too little help, source errors, corrections and localized explanations. Keep [later language rollout](implementation-plan.md#5-extend-only-the-content-the-learner-next-needs) separate from checks using multilingual fixtures.

**Acceptance check** — Report materially wrong learner-facing results, false rejections, useful basic and enriched cards, time and usage by route and tested language. Qualification requires zero materially wrong results on that sample. Keep Gemini free on basic practice unless new evidence supports enrichment on natural drafts and planted errors, including misleading synonyms and changes of meaning. Verify the implementation through the ordered [repository gate](../../AGENTS.md#done-gate). Report content review, software checks and attended audio acceptance separately.
