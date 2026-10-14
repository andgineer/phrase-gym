# Acceptance gaps

Automated checks establish software behavior, not device output, linguistic quality or successful recovery on a deployed host.

- **Device audio/accessibility:** recording works in Safari on MacBook Pro M1. iPhone Safari over private HTTPS records the first take, but successive takes were reported to produce no response until microphone access is requested again. The retained-stream/audio-session correction needs actual-device acceptance; numeric browser/iOS versions and timing measurements were not supplied. Headsets, longer/successive takes, saved-take reopening, background/phone interruptions, pause/resume, VoiceOver and enlarged/non-Latin text still need focused acceptance. Synthetic browser capture cannot certify those behaviors.
- **Content and reference speech:** actor/course clips and prepared Serbian content/speech need semantic/listening acceptance. Development fixtures and positive container checks are not an accepted release language pack. The local 44-phrase pack has one manual cue per phrase; it does not establish alternate-cue preparation quality.
- **Private release:** permanent host/access policy, measured resource fit, consistent backup/restore and upgrade/rollback remain unverified.

Concrete remaining tasks are in [the implementation plan](plan/implementation-plan.md). Keep learner data and recordings private; use disposable data for automation.
