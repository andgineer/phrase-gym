# P1 manual feedback — 2026-10-12

The user reviewed the disposable prototype and requested these changes. This feedback does not approve P1 as a whole or authorize P2.

- **UI language in version one:** allow native-language UI or UI that follows the selected learning language from the first version. Explanations keep using the native language. The prototype exposes the existing preference at first launch as well as under Settings; exact placement remains open to manual feedback.
- **Recording defaults on:** fresh use starts with recording enabled. Capture and microphone permission still begin only on an explicit microphone-button press. The learner can turn recording off; resuming an existing session preserves that session's choice.
- **Show answer:** the former “Additional hint” action reveals the complete phrase. Rename it **Show answer / Показать ответ / Prikaži odgovor** and record full-phrase exposure. Do not imply a subtler hint or add a new hint ladder. Show & explain remains the separate action for meaning and usage. Revealing before a response is supported practice, not independent recall.
- **Remove Other answer / problem:** after hearing the purpose of the alternative-wording/unclear-cue/technical-problem submenu, the user explicitly chose **Remove it entirely**. Remove the action and submenu without substituting a skip/report button. Allowed equivalents still count as correct under the self-check criteria; failed capture still creates no automatic memory-failure grade.

Continue manual prototype feedback with targeted implementation checks. Do not restart the fresh-session review loop. Commit authorized changes directly to `main`.

## Implementation verification

Implemented in disposable prototype version **0.1.2**, with offline shell cache `phrase-gym-p1-v3`. The fixture content is unchanged. Verified on 2026-10-12 with 11 core scenarios, 11 browser walkthroughs and 16 existing regression scenarios, all passing. The new walkthrough covers first-launch UI selection, native/learning UI switching, default-on recording without implicit microphone access, honest full-answer exposure, removal of the problem controls and recording-off persistence on resume. Existing audio capture, storage, offline playback and interruption scenarios still pass. The 320px supported-answer screenshot was inspected.

These are software checks with synthetic microphone input, not actual-device or listening-quality acceptance. Manual prototype review remains the next step; no new fresh-session review was run.
