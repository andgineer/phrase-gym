# Local application availability

In the current local deployment, the browser needs the running Phrase Gym service for library, practice, imports, settings and saved takes. It does not need an internet connection or a running Coach to practise already prepared material. A provider connection is needed only when Coach performs preparation that calls that provider; first-MVP phrase text is externally prepared and reference speech is synthesized locally by Coach. Later cloud placement would require its own access and connectivity design.

If the application stops, show a clear connection message and Retry. Keep active media stoppable, preserve already captured pending takes and avoid confirming unsaved work. On reconnection, resend the same pending writes and reload the confirmed card/session state. Retrying must not duplicate a review or recording.

Retain the useful Vue pending-save behavior; it protects ordinary reloads and brief outages. Do not create an offline scheduling engine, service worker, synchronization service or availability framework. The browser must not advance through unconfirmed scheduled reviews while the application is unavailable.

Coach can be offline when the learner submits preparation work. Phrase Gym durably acknowledges the request and shows its pending state; Coach processes it after returning online. A failed or interrupted preparation affects its submitted batch, not ready cards. The browser does not contact Coach directly or require its availability to submit work. Automatic recovery from every sleep, forced quit or power loss is not a promise; retained work must remain visible and safe to retry without duplicate publication.
