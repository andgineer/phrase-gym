# Language choices

Keep a small Settings page in Vue. The learner can read and change native language, learning timezone and per-language card/help language; no external agent or profile API is required to edit them. Stated proficiency and a short preparation note are optional context. Navigation never changes these choices. Store `daily_due_limit` and `daily_new_limit` settings, initially 20 and five. Daily and Extra read their current values for allowance decisions; an increase can make more work available today. The Settings page does not edit them; a later settings editor may. Each value is a nonnegative integer, and both cannot be zero.

The learner's current choices are:

| Learning language | Stated proficiency | Card language | Short help | Optional detailed help |
|---|---|---|---|---|
| English | B2 | English | English | Russian |
| German | A1 | Russian | Russian | Russian |
| Serbian | Not stated | Serbian | Unresolved; no Russian | Russian |

Native language is Russian; learning timezone is Europe/Belgrade. These are this learner's initial values, not universal defaults. The MVP only needs the English row. Unresolved Serbian help does not block English or German work; resolve it before adding Serbian preparation.

Card language controls retrieval cues; short help explains meaning/use after reveal and during study. Optional detailed explanation uses the native language and appears only on request. Neither answer-bearing layer is shown before an independent response. A detailed explanation and a proficiency questionnaire are not requirements for every MVP card.

Imported batches declare their languages and must match the selected choices; mismatch reports what to change instead of silently translating. Later AI preparation snapshots the choices at submission. Changing Settings affects subsequent preparation, not existing cards or saved recording snapshots. Existing content can be replaced explicitly; no revision registry or automatic mass re-preparation is required.

Keep existing independent device/audio preferences where useful. UI language follows the native or current learning language as already supported by Vue; no new localization project is needed for the first English content path.

[Research on translation and explanations](research/words-in-context.md#translation-or-l2-explanation) does not prove that advanced learners should avoid their native language. Comprehensibility and this learner's choices govern these settings. No source establishes a precise proficiency cutoff for switching prompt language.
