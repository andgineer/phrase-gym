# Acceptance gaps

Automated checks establish software behavior, not device output, linguistic quality or successful recovery on a deployed host.

- **Device audio/accessibility:** recording works in Safari on MacBook Pro M1. The user's iPhone Safari capture/comparison retest succeeds over private HTTPS with native capture defaults and fresh input acquisition per take. Numeric browser/iOS versions, output route and timing measurements were not supplied. Headsets, longer/successive takes, saved-take reopening, background/phone interruptions, pause/resume, VoiceOver and enlarged/non-Latin text still need focused acceptance. Synthetic browser capture cannot certify physical microphone input or device output.
- **Content and reference speech:** actor/course clips and prepared Serbian content/speech need semantic/listening acceptance. Development fixtures and positive container checks are not an accepted release language pack. The local 44-phrase pack still uses one manual gapped cue per phrase; it does not establish the preparation agent's quality.
- **Preparation agent:** unattended runs with the learner's signed-in Chrome are verified for Claude Code only; Codex browser access is unverified. Whether EPAM course pages and their audio need the learner's login is not yet checked.
- **Private release:** laptop access policy, measured resource fit, consistent backup/restore and upgrade/rollback remain unverified.

Concrete remaining tasks are in [the implementation plan](plan/implementation-plan.md). Keep learner data and recordings private; use disposable data for automation.
