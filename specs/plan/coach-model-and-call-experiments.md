# Qualify and implement Coach preparation

Implement the provisional [preparation by provider](../decision-external-preparation-agent.md#preparation-by-provider), starting with the English product path. The multilingual benchmark used synthetic settings. German and Serbian product support still needs the language-specific work in the [delivery plan](implementation-plan.md#5-extend-only-the-content-the-learner-next-needs).

## Content and publication

- Improve cues for bare words and short expressions. A single initial can leave too little evidence to recall a noun and its article or a phrase and its preposition. Check that each cue supports recall while concealing the complete target.
- Render instructions, help and inbox reasons in the configured languages. Check the meaning of each gloss against the complete example. Allow accurate short translations such as Russian **яд** without padding them to satisfy a character minimum.
- Add a basic Coach result to the contract so a confirmed form and a usable cue can reach practice without invented explanations, a second hint or usage contexts. Test both basic and enriched results through the real API and client. Save the matching reference audio and publish atomically, preserving complete external-package validation and existing learner data. Implement the [Coach inbox outcomes](../decision-external-preparation-agent.md#results-in-coach).
- Qualify changed prompts and validators on fresh held-out inputs with independent semantic review. Require zero materially wrong learner-facing results on the sample and check false rejections, useful basic and rich coverage, latency and usage by route and language. If Gemini free enrichment is reconsidered, it needs fresh evidence that the verifier catches misleading synonyms and meaning changes in natural drafts as well as planted errors.

## Explicit models and normal CLI environment

Implement the [provider and CLI policy](../architecture.md#coach-provider-selection) with explicit model and effort, bounded execution, private evidence and existing global CLI settings. Remove Coach-specific tool restrictions. Exercise Autodetect order, pins and CLI overrides, subscription status or bounded access probes, cache invalidation, and free Flash model discovery with fake adapters. Cover quota, timeout during verification and exhaustion of eligible routes. Assert that semantic rejections stay terminal, incomplete drafts cannot publish and retries preserve the pin and publish at most once. Run the repository gate on the implementation.
