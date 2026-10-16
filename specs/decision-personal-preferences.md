# Personal preferences

Personal preferences used by Phrase Gym Coach belong to the inbox domain and are stored in Phrase Gym Server's single database, alongside inbox work and learning data. The learner describes preferences to an external agent, which reads and updates them through the inbox preferences API hosted by the core. No language-specific choice is hardcoded as a universal product rule, and ordinary material preparation never edits preferences.

The profile contains the learner's native language, learning timezone, stated proficiency where known, and per-learning-language card language, help language, translation-with-answer choice and durable preparation guidance. Card and help languages are independent. Missing required preferences remain unresolved; the preparation job reports what is missing instead of guessing or prompting the learner during an unattended run. The external agent resolves them with the learner and writes the answer through the API.

For the current learner, the initial profile is:

| Learning language | Stated proficiency | Card language | Help language | Translation with answer |
|---|---|---|---|---|
| English | B2 | English | English | Off |
| German | A1 | Russian | Russian | On |
| Serbian | Not yet stated | Serbian | Not yet chosen; no Russian | Not yet chosen |

The learner's native language is Russian and learning timezone is Europe/Belgrade. These are personal initial values, not defaults imposed on other learners. Serbian help and proficiency are resolved through the external agent before preparing material that depends on them; Serbian prompts remain Serbian for this learner.

## Updates and preparation

The API validates a complete profile revision and applies it atomically with an expected-current-revision check and idempotent mutation identity. Concurrent stale writes conflict rather than silently overwriting choices. Retain revision history and the originating authorized request so changes can be inspected and explicitly restored as a new revision. Credentials that submit material cannot update preferences unless separately granted that capability.

Each preparation generation pins one profile revision. An accepted update affects later generations; it does not silently modify running jobs or existing cards. A newer profile can explicitly requeue affected work as a new generation. Existing items are re-prepared on request. Compatible changes retain target identity and history; a changed retrieval task requires a new independent check before old recall evidence is presented as evidence for the new conditions. New meanings require new targets.

## Core availability

The core reads the same database records used by the inbox API; there is no preference replica or synchronization service. Prepared content and running jobs retain their pinned preference revision. New sessions use the current learning timezone and UI-relevant profile values while historical attempts retain their original context.

When Coach is offline, the core continues serving practice, accepting material and updating preferences. These operations require the live core, not the worker. The learning frontend does not offer a competing personal-preference editor: the external agent uses the inbox API. Device/UI controls that do not change the personal preparation profile remain core concerns. Selecting a learning language or navigating never writes the profile.
