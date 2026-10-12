# Decision: session scope, phrase lookup and lesson navigation

Accepted user requirements, 2026-10-11. This extends the [targeted-drilling decision](decision-targeted-drilling.md) and [interaction design](design/interaction-design.md). Application implementation and final visual styling remain unapproved.

## Starting screen

Keep one **Start session** action and the two controls below. The later [language decision](decision-languages-and-audio-labels.md) adds a learning-language selector above them, represented in the URL. Lessons, phrase lookup and the session use that language; All lessons means all lessons within it.

1. **Lesson to drill.** A searchable dropdown, initially **All lessons**. The default session draws from all lessons according to the spaced-repetition schedule. Typing characters filters lesson titles by substring: `rap` finds `Building rapport` and any other lesson title containing those characters. Selecting a lesson scopes the session to that lesson; selection itself does not start the session. Do not reintroduce separate learning-new-material and rehearsal modes.
2. **Find a phrase.** Search stored phrases by entered text, with smart matching for meanings, situations and likely search words that need not occur in the phrase itself. For example, `greetings` must find appropriate greeting phrases even when their wording contains no such word. Selecting a result opens **Show & explain** directly. This is reference lookup for remembering a phrase, not a review session.

The two controls serve different purposes. The lesson selection controls Start session. Proposed presentation: phrase lookup searches across lessons in the selected learning language independently of the lesson filter; show each result's phrase and lesson title so its context is clear. This independence, labels and result layout are recommendations to review, not additional explicit user choices.

## From a phrase to its lesson

- Show & explain includes a visible link to the phrase's lesson, usable whether the card was opened from search or from a drill.
- The link opens a lesson view listing **all phrases in that lesson**, not just phrases currently due.
- That lesson view includes an action to **start drilling this lesson**.
- Proposed list interaction: selecting any phrase opens its Show & explain card. Preserve the route back to the lesson list or search results; a card opened from a drill returns to that drill with support/exposure retained.

Show & explain retains the full phrase, native-language meaning/use, model audio and accepted optional recording with the model visible. Lookup does not require starting a scheduled drill. If the learner chooses to record from lookup, retain the take under the existing all-recordings policy and classify it as model-supported practice.

## Evidence and scheduling implications

Opening a search result, reading an explanation or browsing the lesson's phrases is not a completed review or a failed recall attempt. It must not generate a success grade or advance a due date simply because the learner looked at the phrase. Preserve actual model/explanation exposure separately, including when browsing precedes a drill. A later masked response does not erase that exposure. Keep one target and schedule when the same phrase is linked from multiple lessons or routes; a lesson filter is not a separate schedule.

The user explicitly selected spaced repetition for the default **All lessons** session. Whether an explicitly selected lesson drills only due phrases or all its phrases immediately is still open; an optional clarification was asked. Do not silently settle that policy through fixture ordering. Session length, queue mixing and handling a lesson with no due phrases also remain open.

## Search preparation proposal

The accepted requirement is useful matching beyond literal phrase text. Its implementation is not yet selected. A proposed starting point is to prepare and store reviewed topics, communicative intentions and likely synonyms alongside phrases during intake, then search wording and this metadata. An LLM could prepare the metadata alongside the already agreed cue variants. Evaluate representative queries before deciding whether embeddings or another retrieval method are needed. This proposal does not authorize arbitrary generated search answers: results refer to existing stored phrases and their lessons.

Case-insensitive title/text matching, relevance ranking and keeping the query when returning from a result are recommended interaction details. Search in the learner's native language would be useful but has not yet been explicitly selected. A no-match state should preserve the query and allow editing; do not fabricate a stored phrase.

## Discussion sketch and future checks

The navigation sketch uses illustrative lessons and phrase text, with a small hand-authored intent/synonym index to demonstrate `greetings`. It demonstrates navigation, not a validated semantic-search engine, real scheduled queue, recording system or database. The fixture's lesson-start action opens an example drill without settling due-only versus all-phrase selection. It does not complete P1.

Future walkthroughs should cover `rap` title filtering, clearing back to All lessons, literal phrase fragments, `greetings` without a literal match, no results, search → Show & explain → lesson → all phrases → lesson drill, and return navigation with query and exposure retained. Verify that browsing alone leaves review schedules unchanged. Exact screen styling, search algorithm and selected-lesson review policy remain open.
