# Phrase Gym

Phrase Gym is a local Python application with a browser interface for studying useful English
phrases and keeping private recordings. Import a prepared phrase package, listen to its reference
audio, record optional takes and replay them after reloading the app.

First study is ungraded. Later recall, scheduling and library controls are still being implemented;
the current app does not claim to measure fluency or schedule a later recall session.

The server works on macOS and Linux. The MVP media player and recording experience target macOS
browsers. Linux browser audio qualification is planned after the MVP.

Start with [installation](installation.md), then [import a phrase package](importing-phrases.md).
The service listens only on loopback and keeps learner data in its local application-data directory.
