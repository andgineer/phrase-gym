# Phrase Gym

Phrase Gym is a local Python application with a browser interface for studying useful English
phrases and keeping private recordings. Import a prepared phrase package, listen to its reference
audio, record optional takes and replay them after reloading the app.

**Daily** offers due recall and new phrases within daily limits. First study is ungraded; on later
learning days the app offers spoken recall with an honest self-rating and a scheduled return.
**Extra** extends today's limits for another pass without erasing completed practice. The entry
shows today's completed practice, the phrases studied today, a factual practice streak and the next
return date. The Library finds phrases by lesson or meaning, shows their details and saved
recordings, practises one chosen phrase, and deletes a phrase after confirmation. The app does not
claim to measure fluency.

The server works on macOS and Linux. The MVP media player and recording experience target macOS
browsers. Linux browser audio qualification is planned after the MVP.

Start with [installation](installation.md), then [add your phrases](importing-phrases.md).
The service listens only on loopback and keeps learner data in its local application-data directory.

For your external AI agent, pass along the [instructions to extract and add phrases](agent-add-phrases.md).
See also [private access](private-access.md), [backup and restore](backup-and-restore.md),
and the [HTTP API reference](reference.md).
