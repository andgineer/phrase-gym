# Local application availability

The browser needs the running local Python application for library, practice, imports, settings and saved takes. It does not need an internet connection to practise already prepared material. A provider connection is needed only for preparation that calls that provider; the first MVP imports externally prepared text and synthesizes reference speech locally.

If the application stops, show a clear connection message and Retry. Keep active media stoppable, preserve already captured pending takes and avoid confirming unsaved work. On reconnection, resend the same pending writes and reload the confirmed card/session state. Retrying must not duplicate a review or recording.

Retain the useful Vue pending-save behavior; it protects ordinary reloads and brief outages. Do not create an offline scheduling engine, service worker, synchronization service or availability framework. The browser must not advance through unconfirmed scheduled reviews while the application is unavailable.

Preparation failure affects its submitted batch, not ready cards. There is no separate Coach whose availability determines whether new work can run. Automatic recovery from every sleep, forced quit or power loss is not an MVP promise; show an interrupted import with an explicit retry.
