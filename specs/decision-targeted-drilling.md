# Decision: targeted phrase and word drilling

Accepted direction, 2026-10-10. This records the user's decisions; it does not approve the entire first-version feature set. See the [functional description](functional-description.md) for the app around these drills and the [remaining questions](plan/functionality-discussion.md).

## Product scope

Train selected words in useful phrases, fixed expressions and specific dialogues until the learner can use them readily in the intended situations. Open-ended AI conversation, free role-play and a general AI speaking partner are outside the product's scope, including as a later direction.

AI may improve the material and the drilling process. Every exercise must have a defined learning target.

## Recall needs a sufficiently specific cue

Introduce the complete expression, its meaning and its use before asking the learner to recall it. During a recall exercise, make clear that the learner is retrieving a previously learned expression.

A broad instruction such as “Ask how his day was” cannot identify one exact answer. Many responses are good English. A situation or partner line supplies the purpose; phrase-specific material identifies the expression to retrieve.

For each target:

- Choose the smallest useful set of **meaningful chunks** that makes the intended expression clear in that context. These may come from the beginning, middle, end or several places. There is no default “show the first words” rule.
- Consider plausible competing expressions and completions. Keep enough distinguishing material to avoid making the learner guess which wording the app wants. Minimum character count is not the objective.
- Include necessary disambiguating material in the initial prompt. It must not be withheld until the learner fails or asks for help.
- If the cue still permits several natural answers, strengthen it or explicitly allow those answers. Do not count ambiguity in the prompt as a learner error.

This is practical disambiguation within the context and learned material, not a claim that a partial sentence has only one possible completion in the entire language. Visual and audio forms of a cue must meet the same requirement.

## Vary the hidden part deliberately

Support several cue versions for the same expression. Across repetitions, the app can hide different meaningful parts so the learner recalls more than the same missing fragment every time. Retain enough information to identify the target in every version.

This is controlled variation, not random gaps or first-letter puzzles. Reusing a useful cue is allowed; a new mask is not required on every attempt. The frequency and adaptation rules remain open.

An illustrative example, after learning **“How has your day been so far?”**:

| Attempt | What the learner sees | What the learner says |
|---|---|---|
| Introduction | The full phrase and its use: asking about the listener's day up to now; model audio | The full phrase, following the model |
| Recall with one cue | “Recall the expression you learned.” Same purpose, plus **“_____ your day been so far?”** | “How has your day been so far?” |
| A later cue version | Same task and purpose, plus **“How has _____ been so far?”** | “How has your day been so far?” |

These examples illustrate the behaviour, not a universal mask template. **The learner always says the whole phrase**, not only the missing words. Each actual cue needs checking against plausible alternatives.

## Practice and feedback

1. **Introduce:** hear and see the full expression in a useful sentence or short dialogue; understand it. Listening and echoing can help establish the model.
2. **Retrieve:** receive the situation or partner line together with a sufficient phrase-specific cue; say the complete expression aloud.
3. **Compare and repair:** compare with the model. When stuck, request additional meaningful material or the full model, then repeat the repaired expression.
4. **Return:** revisit the same target after intervening items and on later days. Adjust support to performance and use different hidden chunks when useful.
5. **Use in a specific dialogue:** learn the dialogue first, then rehearse the learner's side with target-specific cues. Isolate a difficult exchange and restore it to the dialogue. Controlled changes may exercise the same word, frame or expression in another appropriate situation.

This describes available practice, not a compulsory ladder with fixed pass thresholds. Gradually reduce support where the task remains clear. A broad prompt alone must not become an exact-phrase test just because the learner has progressed. Seek accurate, increasingly ready speech; no universal response countdown is decided.

## What counts as success

- **Fixed expression:** retrieve the learned wording, with explicitly allowed equivalents such as appropriate contractions.
- **Word or reusable frame:** retrieve the selected sense or construction and use it in the requested phrase. Define what stays fixed and what can change; do not demand an unrelated model sentence verbatim.
- **Valid alternative:** distinguish good English from retrieval of the requested target. If the cue was insufficient, repair the exercise. Do not infer forgetting merely because the learner chose another expression.
- **Support matters:** reading a full model, immediate echoing, partial-cue recall and independent recall are different evidence. A visible target word can support practice of its use, but does not demonstrate recall of that word.

One learning target has one review schedule across its cue versions, contexts and dialogue appearances. Do not create independently scheduled sibling cards for different masks. Record the support used; the scheduling algorithm and progress thresholds remain open.

## AI's role and the evidence boundary

AI may prepare useful word-in-phrase examples, short dialogues, candidate cue chunks and alternate masks. It can check plausible competing completions and help refine an ambiguous prompt. Generated material needs checking for naturalness, meaning and suitability for the target; an AI check is not proof of uniqueness.

Optional assistance during practice may identify an omitted target, offer a better cue or select a repair exercise. Uncertain speech recognition must not be treated as a definite learner mistake. Self-comparison with the model remains available; automatic assessment is not a prerequisite for the core drill.

Research supports deliberate retrieval and spaced practice, and includes phrase-learning tasks whose cues constrain the intended answer. **Phrase-specific chunk selection, rotating masks and this complete spoken workflow are product decisions to evaluate**, not a package already established as best by research. See the [focused evidence record](research-memorizing-phrases.md).
