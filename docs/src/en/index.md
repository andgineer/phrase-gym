# Phrase Gym

Phrase Gym is a local Python application with a browser interface for studying useful English
phrases and keeping private recordings. Type a phrase or word for Coach to prepare, or let an AI
agent add a course batch, then listen to reference audio, record optional takes and replay them
after reloading the app.

The **Practice**, **Library**, **Coach** and **Settings** tabs switch pages. The brand returns to
Practice, and the installed app version stays visible on every page. Open **Coach**, type a phrase
or word and optionally its source sentence, then press **Send** or Enter. Pasted line breaks become
spaces. Coach prepares the card later for **Added with Coach** in Library. A Library search offers
to send missing material to Coach, with the same confirmations. Switching tabs finishes and keeps
any recording, and **Start** resumes the regular card. Reloading keeps the selected page.

**Start** offers the longest-due phrases and randomly chosen new phrases within the daily limits set in
**Settings**, mixed in random order so the next phrase cannot be anticipated. Every phrase starts with its answer hidden,
including a new phrase. Press **Reveal** and rate the attempt; an unknown answer is **Again**. The first
rating starts its schedule, and later learning days offer due recall. Each rating button shows when
that grade brings the phrase back, such as `3d` or `2.5m`; it shows nothing when the phrase was
already graded that day. **Settings** chooses whether phrases come back **Often**, **Balanced**
(the default) or **Rarely**, with how many times a phrase you keep answering **Good** returns in
its first month and year.
The Start card shows the lesson's due and new phrases within today's remaining allowance. Start stays
disabled when none wait; **Extra** appears only if eligible phrases remain beyond the allowance, and
extends today's limits for another pass without erasing completed practice. The entry
shows today's completed practice, the phrases studied today, a factual practice streak and the next
return date. An optional weekly target in **Settings**, one to six practice days from Monday to
Sunday, adds this week's practice days beside the streak; it can be changed or paused at any time,
a missed day or week carries nothing forward, and it never changes schedules or daily limits.
The Library finds phrases by lesson or meaning, shows their details and saved recordings,
practises one chosen phrase, and deletes a phrase after confirmation. The app does not claim to
measure fluency.

The full-width microphone stays in the same place on both sides, above the text. Ratings and Skip
stay at the bottom while the text scrolls. The back’s **More actions** menu offers **Delete phrase…**
with confirmation. Tap the microphone to record, then tap it again to finish. Finishing on the front reveals the
answer and plays the reference followed by your recording; recording on the back uses the same
playback order. Once you press **Ref** on the back, finishing each later take on that card
plays only your recording. The history icon in the audio controls on the card back or in phrase details opens a
framed list of saved takes, newest first, with a date and **Mine** replay button for each.
It stays collapsed until you open it.

The server works on macOS and Linux. The MVP media player and recording experience target macOS
browsers. Linux browser audio qualification is planned after the MVP.

Start with [installation](installation.md), then [add your phrases](importing-phrases.md).
The service listens only on loopback and keeps learner data in its local application-data directory.

See also [private access](private-access.md), [backup and restore](backup-and-restore.md),
and the [HTTP API reference](reference.md).
