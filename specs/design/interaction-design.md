# Phrase Gym — interaction design discussed in this session

Updated 2026-10-11. This is a written design for review and future prototyping, not an implemented or visually approved app. The [drilling decision](../decision-targeted-drilling.md) and [recording decision](../decision-recording-and-comparison.md) define agreed behaviour. The final visual style and complete navigation remain open.

**Latest answer-card recommendation:** following the user's academic-research request, [Not yet / With effort / Recalled](answer-card-and-review-policy.md) replaces Got it / Try again as the assistant's recommended grading design. The active flow below reflects it; older sketches retain their historical labels. Keep extra practice separate from the original result and give ambiguous cues/technical failures a no-grade route. The user explicitly rejected effectiveness-validation work: no in-app experiment, special assessment session or calibration task.

## Main practice screen

The main screen has one **Start session** button. Learning new material and rehearsal are not separate home-screen entry modes. Phrases are expected to have been encountered in source material, but the drill must support forgetting and lack of understanding without requiring the learner to leave the session.

**Accepted language addition:** provide a learning-language selector, with its selection in the URL for bookmarks. Recommended header: `Phrase Gym    English ▾`, above lesson selection and phrase lookup. Those controls operate within the selected learning language; All lessons no longer implies a queue spanning all studied languages. Example proposed routes: `/learn/en` and `/learn/sr`. See the [language/audio decision](../decision-languages-and-audio-labels.md).

**UI language is separate:** buttons, menus and status text use either the learner's native language or the current learning language. Explanations remain in the native language; phrases, situations and partner lines remain in the learning language. Recommend one settings preference, Use learning language for the interface, instead of an independent language dropdown. Preference presentation/default and route syntax are proposals. English labels in this specification are working copy to localize.

**Accepted starting-screen controls:** a searchable lesson dropdown defaults to All lessons according to spaced repetition. Entered characters filter lesson titles (`rap` → Building rapport and other matching titles). A separate phrase search matches both entered phrase fragments and concepts absent from the wording (`greetings` → appropriate greetings). Selecting a phrase opens Show & explain for reference, without starting/completing a review. See the [navigation/search decision](../decision-session-navigation-and-search.md).

**Accepted lesson route:** Show & explain links to the phrase's lesson. Its lesson view lists all phrases and offers Start lesson drill. Proposed return behavior preserves the search query or lesson list, while a card entered from a drill returns to that drill. Search results show phrase wording plus lesson title. These placement/return details are proposals; due-only versus all-phrase drilling for a selected lesson remains open.

Recommended layout: one expression at a time, a compact situation/partner line above a prominent phrase cue, one primary response action and visible help controls. Keep the situation with the cue so the learner knows which learned expression is intended. The user accepted a compact drill and rejected repeated instructional/readiness copy.

Illustrative recording-on screen for a known expression:

```text
‹ Pause                         3 of 8
Recording: On

A colleague's day
You meet a colleague during a break.
Ask how their day has been up to now.

        […] your day been so far?

          [microphone icon]

           Show & explain
```

The learner says “How has your day been so far?” in full. The missing chunk is not the whole response. The sample count, labels and screen spacing are illustrative, not final design decisions. **Show & explain** is the current proposed label for the agreed action that reveals the full phrase and native-language explanation. Each real cue requires its own review against competing completions. Additional meaningful cue help can remain available where useful; this compact sketch does not settle its placement.

**Accepted recording presentation:** omit “Say the whole phrase aloud.” and “Ready when you are.” The idle recording button contains only a microphone icon, with an accessible Start recording name. During capture, replace it with a stop icon and the visible label **Stop recording**. That action still triggers the agreed automatic learner/model comparison. Keep support/exposure in history without adding routine instruction text to the ready card. Useful playback status and actual error messages remain available.

## Recording-on interaction

| Step | What the learner sees/hears | What the learner does |
|---|---|---|
| Cue | Compact situation/partner line, identifying phrase chunks, microphone-icon button. | Read/hear the cue and press the microphone button. |
| Recording | The same cue; the primary button now has a stop icon and Stop recording label. | Say the whole phrase, then tap Stop recording. Manual stop is the first-version behaviour and the default. |
| Own version | Automatic playback of the just-recorded take; keep the initial cue visible. | Listen; interrupt playback if desired. |
| Reference | Reveal the complete phrase and automatically play its reference audio after the learner take. It may be an imported actor/course recording or generated speech. | Compare the two versions. |
| Self-check | Recommended: Not yet, With effort, Recalled; replay controls, Practice again and Other answer / problem remain available. | Rate the original recall: incorrect/missing, correct with substantial effort, or correct without substantial difficulty. Extra practice is a separate choice. |
| Continue | Recommended: a result tap saves the observation and advances, with Undo. Supported practice uses Continue without a recall grade. | Explicit learner action advances; playback completion never grades or advances automatically. If extra practice was requested, retain the original result before making another take. |

Comparison controls:

```text
     How has your day been so far?

      ▶ My recording    ▶ Reference

        How did you recall it?
     Not yet   With effort   Recalled

      Practice again   Show & explain
           Other answer / problem
              What counts? ▾
```

Recording is optional for the session and the switch is accessible during practice. Recording start is an explicit learner action. **Manual Stop recording is the accepted default and the only stopping method in the first version.** Choosing manual versus automatic stop after a pause belongs in later app settings; the selector and automatic stopping are deferred beyond the first version. Do not silently turn “automation” into automatic grading or a permanent open microphone.

The user rejected **Model** as an audio-button label. **Reference** is the recommended working replacement throughout the drill and explanation card, with the accessible name Play reference audio. Its wording does not imply a particular audio source. Earlier sketches and pedagogical references to “model” must not override this correction.

All takes are automatically retained in the app database from the first version, including rerecordings. There is no Save recording step; recording another try does not replace earlier audio. Preserve target, date/time, cue and support context for a future progress-comparison interface. That historical interface is deferred; the immediate self/model replay controls above remain core.

## Recording-off / cannot-speak interaction

Show the same phrase-specific task and use **Reveal & compare** as the primary action. The learner can recall silently, reveal the full phrase and request model audio manually. No microphone permission is needed. Keep help, repair and alternative/unclear-cue reporting available. Recommended: deliberate silent retrieval uses the same three memory ratings, with modality retained; reading/recognition alone is exposure. No spoken-performance result is inferred.

The session switch controls recording; a future explicit unrecorded-spoken mode would require clear wording and evidence handling. Do not claim that disabling a microphone proves whether the learner spoke.

## Explanation card and return to the drill

**Accepted behaviour:** expect prior source exposure and begin with the drill; do not force an in-app introduction merely because a phrase is new to the database. The learner may still have forgotten or never understood it. A visible action opens a learning/explanation card showing the complete target-language phrase and explanation in the learner's native language. Native-language translations/explanations are absent from the ordinary drill; UI controls may nevertheless use that language under the interface preference. This replaces the earlier mandatory new-item introduction and separate home learning/rehearsal proposal.

**Compact presentation is the chosen direction. Proposed Show & explain layout:** keep the current phrase and session context; replace the drill body with the full target-language phrase, Reference and practice-recording controls, a concise native-language meaning and usage explanation, and an optional expandable phrase breakdown/example. Earlier sketches place Listen and the microphone control together below the complete phrase; use Reference as the current working label. A Back to drill control returns to the same cue. Exact placement, detailed content and navigation presentation remain under review.

**Accepted recording on this card:** the learner can record while the full phrase remains visible. Microphone → Stop recording → automatic learner playback → model playback stays on the explanation card, with the phrase visible throughout. Replay either version or make another take as needed. The session recording switch applies here too; recording-off mode permits explanation and manually requested model audio with no microphone access. Every recorded take is stored and marked as practice with the model shown, linked to the same target/history. The current sketch demonstrates these states with simulated audio and capture.

Opening explanation alone does not imply failure. Record which full model, explanation and audio were exposed and when. Return to target-language-only practice with that help retained in history; immediately hiding the phrase again does not make the next response independent recall. Help requested after an attempt does not retroactively change that attempt's support. Recommended: after help before any attempt, show a supported-practice state with Continue / Practice again, without independent-recall grading. Revisit after intervening material and time under the [repair policy](answer-card-and-review-policy.md#concrete-session-rules). A second successful take does not erase earlier difficulty or help. Persistent difficulty returns to meaning or cue repair.

Necessary distinguishing material still belongs in the initial cue. Additional meaningful hints may be offered where useful; the full phrase/explanation route is always available without first failing or exhausting hints.

## Native language at first launch

Accepted on 2026-10-11: when the app first starts with an empty database, ask for the learner's native language and persist it as an **app setting**. Use that value for phrase explanations. Reuse the saved setting on subsequent starts; do not ask on each drill or each session. This is separate from the language of the learning material. Russian is the user's selected language for the current sketch; do not hard-code it for every learner. The first-launch screen and broader settings layout are not yet designed.

When adding a phrase to the database, the LLM prepares alternate versions hiding different words or meaningful chunks where the phrase supports them. Checked variants are stored with the phrase, allowing a later session to present a different cue. The learner still says the same whole phrase. Reuse a single reliable cue when no useful alternative exists; cue variation must preserve clarity. See the [accepted preparation decision](../decision-targeted-drilling.md#prepare-variants-when-adding-the-phrase).

## Different wording and unclear cues

The earlier compact outcome proposal was **Got it**, **Try again**, **Different wording**. The current research-informed recommendation is **Not yet**, **With effort**, **Recalled**, with Practice again separate and an Other answer / problem route. See the [answer-card design](answer-card-and-review-policy.md) for exact scoring and supported-practice states; these labels remain a recommendation.

Good alternative English is not automatically a memory failure. An unclear cue needs correction. For fixed phrases, show explicitly allowed equivalents; for words/frames, show which parts may vary. Keep one target/result history across changed masks and recordings.

## Specific dialogue rehearsal

Learn the complete short exchange first. Rehearse the learner's side: partner line → target-specific cue → full response → comparison. Leave supporting lines readable. Recording-on turns follow the same own-take/model sequence; recording-off turns remain usable silently. Isolate a difficult target turn, repair it, then reinsert it at its original point without creating another learning target or schedule.

## Visual direction and remaining design work

The assistant recommended a calm, focused style: large phrase text, warm neutral background, one restrained accent and generous spacing. A more energetic gym-like style with stronger colours/progress was offered as an alternative. **The user did not select either visual style.** No palette, typography, navigation, home screen, progress dashboard, carousel choice or earlier HTML sketch is accepted by implication.

Before implementation of unresolved screens, continue the design discussion on those choices. P1 should let the user inspect the concrete seven practice scenarios, both recording modes, automatic comparison, permission/failure states, pause/resume and honest results. The deliverable is a reviewed prototype and actual-device evidence, not an assumed visual approval.

## Resumed prototype discussion 2026-10-11

**User direction:** continue discussing app prototypes and functionality; preserve all decisions in `specs/`. This resumes design review, not application implementation. Accepted choices belong in the relevant decision documents as the discussion progresses. Suggestions, unchosen alternatives and unresolved questions retain their status.

**Discussion sketch:** a single mobile practice screen for the already introduced expression “How has your day been so far?” demonstrates the existing recording-on and recording-off paths. The learner can try Record → Stop & compare → simulated learner playback with the cue retained → full phrase and simulated model playback → self-check. Turning recording off shows Reveal & compare and manually requested model playback. Full-model help, supported retry and the different-wording/unclear-cue branch are also illustrated.

The illustration uses manual stop, which the user subsequently accepted as the default and the only first-version stopping method. The warm neutral surface, green accent and large phrase text remain **visual proposals, not accepted decisions**. Recording starts on only to expose the recording flow in the illustration; this does not decide the app's recording on/off default. The compact example uses Show phrase; additional meaningful help remains part of the wider design when a useful intermediate cue exists. Its exact labels and interruption handling still require review.

The conversation sketch is stored at `/Users/andrei_sorokin2/.codex/visualizations/2026/10/11/01a12a6f-c63e-7d32-a9c9-be0bbc8da9ae/phrase-practice-walkthrough.html`. This document preserves its intent without requiring that file to survive. It simulates recording and playback, with illustrative timing, and does not access a microphone, play actual audio, store learning history or validate a device. It is not the runnable P1 app prototype or evidence that any P1 gate has passed. Clicking through it is not approval.

**Accepted choice:** the user stated, “this is in app settings (not implemented in 1st version), by default manual stop”. Use manual stopping in the first version, with the subsequently selected Stop recording label. Add the choice of stopping method to app settings in a later version, retaining manual as the default. Automatic stop is deferred; detection timing and device behaviour remain untested. This does not defer the existing session-level recording on/off switch or approve the rest of the sketch. See the updated [recording decision](../decision-recording-and-comparison.md#stopping-the-recording-manual-by-default-settings-choice-deferred).

**Accepted recording retention:** the user chose to keep all recordings in the database now, allowing a future UI to compare progress. Automatically store all audio and associated practice metadata, including retries, without a manual Save step or automatic expiry. Historical comparison UI is deferred. This supersedes the assistant's selective-saving proposal; see the [recording decision](../decision-recording-and-comparison.md#store-all-recordings-progress-comparison-ui-later).

**Accepted cue clarification:** the user requested differently hidden words when possible for a phrase, prepared by the LLM when the phrase is added to the app database, so the next session can show another version. Preparation timing, storage for later reuse and conditional variation are now explicit in the drilling decision and plans. The subsequent recording-retention decision is recorded above.

**Accepted unified session and help:** the user requested one Start session button, with no separate learning/rehearsal entry modes. Prior source exposure is expected, but every drill must accommodate a forgotten or misunderstood phrase through full-phrase/native-language explanation help. They then selected Russian for the sketch and required native-language selection on first launch with an empty database, saved in app settings.

**Current discussion sketch:** `/Users/andrei_sorokin2/.codex/visualizations/2026/10/11/01a12a6f-c63e-7d32-a9c9-be0bbc8da9ae/drill-and-explanation.html` opens on the Russian explanation card, now including accepted practice recording with the full phrase visible and automatic learner/model comparison. The session recording switch is accessible from both views. Returning to the drill shows the compact context and cue, microphone-icon start control, stop-icon/Stop recording capture state and Show & explain action. The two rejected helper lines and the repetitive flow footer remain removed. It is an interactive discussion illustration, not application implementation: no real audio, recording, database writes or P1 gate completion. The prior practice sketch remains historical. Compact presentation and recording on both cards are accepted; overall styling, explanation detail and exact control placement remain proposals.

**Explanation-card recording choice — accepted:** the user answered yes to the proposal for optional recording/repetition with the full phrase visible, using the same learner/model comparison and recording the support used. The sketch and [recording decision](../decision-recording-and-comparison.md#recording-on-the-explanation-card) now include it; the read/listen-only alternative is no longer the chosen design. The proposed explanation contents remain a natural Russian meaning, short usage context and optional focused breakdown of useful chunks/constructions (for example, “so far”); exact content depth is still open.

The discussion has now moved to home selection, phrase lookup and lesson navigation, recorded below. Recording on/off defaults, selected-lesson review eligibility and exact queue mixing remain open; the native-language setting requirement is settled.

Sketch checks: JavaScript syntax and referenced controls were checked. A standalone browser preview was not verified: the local preview server could not bind in the sandbox, and browser policy rejected a local-file URL. No device, visual-approval or production-functionality result is claimed.

## Home, lookup and lesson navigation — 2026-10-11

The user requested the two home controls and lesson route described above. The [dedicated decision](../decision-session-navigation-and-search.md) preserves the accepted behavior and separates search-engine, ranking and selected-lesson policy proposals. Lookup and list browsing must not advance schedules; retain exposure to full phrases without reporting it as an independent retrieval attempt.

The latest navigation sketch is `/Users/andrei_sorokin2/.codex/visualizations/2026/10/11/01a12a6f-c63e-7d32-a9c9-be0bbc8da9ae/session-and-lookup.html`. It starts on Home, supports title filtering with `rap`, literal phrase search and illustrative intent matching with `greetings`, then Show & explain → lesson → all phrases → Start lesson drill. Each sample phrase has its own Russian explanation. Back navigation retains the query; explanation recording and compact drill controls remain available. The earlier `drill-and-explanation.html` remains the focused card sketch.

These navigation/earlier card sketches retain the former self-check labels. The focused `answer-card-research.html` illustration described in the [current answer-card proposal](answer-card-and-review-policy.md#discussion-illustration) is the latest grading sketch. Neither sketch implements a scheduler; the active written flow above takes precedence over obsolete labels.

The sample search uses six authored phrases and intent terms across three illustrative lessons. Starting a lesson opens one example drill; it does not demonstrate a real due queue or settle selected-lesson eligibility. Audio/capture are simulated and no database writes occur. Visual styling remains proposed. Static syntax/control checks and local state-transition checks are distinct from actual browser/device verification; the earlier browser-preview limitation still applies.
