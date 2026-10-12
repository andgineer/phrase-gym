# Decision: learning language, interface language and reference audio

2026-10-11. Accepted user requirements from the ongoing design discussion. Exact route syntax, preference presentation and the replacement audio wording below are recommendations. This does not authorize app implementation.

## Reference audio

The user rejected **Model** as an audio-button label because it can imply AI-generated speech. The intended meaning was a reference recording for comparison. Audio may be imported from a course and spoken by professional actors, or prepared with speech synthesis; both sources are supported.

Recommended working copy: **Reference**, paired with **My recording**. Use **Play reference audio** as its accessible action name and **Playing reference** for playback status. Localize these labels with the UI. Keep the same label regardless of audio origin; preserve the actual source/performer or generation metadata separately. Imported actor audio must not be replaced with generated speech merely for consistency.

The accepted learner-take → reference playback order is unchanged. Earlier specifications use “model” in its pedagogical sense (example answer); that terminology is not a claim about AI provenance and must not restore the rejected UI label. Existing HTML illustrations may retain historical wording until their next revision.

The user subsequently requested inspecting `../echo-words` for multilingual audio generation. Its [implementation findings](reference/echo-words-audio.md) are recorded as guidance for missing reference clips: per-language voices, Serbian script preparation, caching and language-correct fallback. Imported course/actor audio keeps priority. No new engine setting was explicitly selected by that inspection request.

**First-version scope confirmed, 2026-10-12:** switching the UI to the learning language is required in the very first version. P1 now offers the toggle during first launch and in Settings. See [manual feedback](decision-p1-manual-feedback.md).

## Three separate language roles

| Role | Required behaviour |
|---|---|
| **Native language** | Ask on first launch with an empty database and save in app settings. Use it for meaning/use explanations. Changing the learning language does not change it. |
| **Learning language** | Provide a visible selector so the same learner can study several languages and switch between them. Encode the selection in the URL so each language can be bookmarked. |
| **Interface language** | May be the native language or the currently selected learning language. It is not necessarily the explanation language. Do not add an independent dropdown of arbitrary UI languages. |

The user learns three foreign languages concurrently; English and Serbian are explicit bookmark examples. Multi-language switching is required, not a hypothetical later expansion. The exact initial content/translation inventory remains a release detail.

“Normal drills use the target language” refers to the phrase, situation and partner-line content. Navigation, action buttons, settings and errors follow the interface preference. Explanations use the native language even when their surrounding controls use the learning language.

Recommended preference presentation: one settings toggle, **Use learning language for the interface**. Off uses the native language; on follows the selected learning language. This is a proposed way to express the two permitted modes, not an accepted extra language selector. Recommend native-language UI initially; the default and toggle placement have not been explicitly chosen. A global mode suffices; derive the actual UI language from each tab's learning-language URL.

Example with native Russian: the English view can have English controls and Russian explanations. Switching to Serbian changes the phrases and, when the preference is enabled, the controls to Serbian; explanations remain Russian. With the preference off, both views use Russian controls and explanations.

## Language selection and bookmarks

Recommended routes and titles:

| Learning language | Example relative URL | Browser/bookmark title |
|---|---|---|
| English | `/learn/en` | Phrase Gym: English |
| Serbian | `/learn/sr` | Phrase Gym: Serbian |

These illustrate the proposed route format, not existing deployed addresses. Localize the language name in the browser title with the UI; stable route codes do not change with translation.

Recommended navigation rules:

- Place the learning-language selector in the app header, separate from lesson selection and phrase lookup. Selecting a language updates the URL and active language view without starting a drill.
- Scope lessons, phrase search and Start session to the selected learning language. **All lessons** means all lessons in that language; phrase lookup can still search across its lessons independently of the chosen lesson filter.
- Preserve the language in phrase/lesson routes. Direct links and reloads resolve from the URL. Explicit URL language takes precedence over a stored last-used language, including when English and Serbian are open in separate tabs.
- On a bare app URL, use the last valid learning language if available; otherwise show the language choice. An empty or unavailable language gets an explicit empty/selection state, not another language's content under the wrong URL.
- Keep one database and each target's own schedule/history. Switching languages does not reset progress, duplicate targets or mix languages into one review queue. Changing UI language does not translate or replace the learning targets.
- Preserve an in-progress session when switching language. Proposed behaviour: pause it and expose Resume on return; never relabel its language or discard recordings. Finish/stop active capture before navigating. Viewing a different language in another tab does not itself start or take over a session; existing single-active-session rules still apply.

Route mechanics, the preference control/default and session-resume presentation are recommendations for the next sketch. No learning-effectiveness test or new assessment feature follows from these changes.

The accepted [device-language and Settings decision](decision-device-language-and-settings.md) specifies the current restoration policy: best-effort browser storage, no server learning-language preference, and one pending Settings save. It supersedes earlier server preference synchronization requirements.
