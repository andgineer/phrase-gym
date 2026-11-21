# Local application availability

In the current local deployment, the browser needs the running Phrase Gym service for library, practice, imports, settings and saved takes. It does not need an internet connection or a running Coach to practise already prepared material. A provider connection is needed only when Coach performs preparation that calls that provider; first-MVP phrase text is externally prepared and reference speech is synthesized locally by Coach. Later cloud placement would require its own access and connectivity design.

If the application stops, show a clear connection message and Retry. Keep active media stoppable, preserve already captured pending takes and avoid confirming unsaved work. On reconnection, resend the same pending writes and reload the confirmed card/session state. Retrying must not duplicate a review or recording.

Home-screen installation opens the same server-backed client in standalone mode. It adds no service worker or offline cache.

Retain the useful Vue pending-save behavior; it protects ordinary reloads and brief outages. Do not create an offline scheduling engine, service worker, synchronization service or availability framework. The browser must not advance through unconfirmed scheduled reviews while the application is unavailable.

Coach can be offline when the learner submits preparation work. Phrase Gym acknowledges the saved request. Coach processes it after returning online. A failed or interrupted preparation affects its submitted batch, not ready cards. The browser does not contact Coach directly or require its availability to submit work. Retain interrupted work and make transport retries safe against duplicate publication. [Coach results](decision-external-preparation-agent.md#results-in-coach) distinguish rejected input from service failures.
