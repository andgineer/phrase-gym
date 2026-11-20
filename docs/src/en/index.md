# Phrase Gym

Practise saying useful English phrases from your lessons, videos, and conversations. Each card
starts with a situation and a hidden answer. Say the phrase from memory, hear the reference,
and rate your recall to schedule the next review.

Start with [installation](installation.md), then [add your phrases](importing-phrases.md).
The **Practice**, **Library**, **Coach**, and **Settings** tabs switch pages. The Phrase Gym name
returns to Practice. Reloading keeps the selected page, and every page shows the installed version.

## Daily and extra practice

On the **Practice** page, choose **All lessons** or select a lesson. You can type to narrow the
lesson list. The due and new counts show what is available within today's remaining limits.
**Start** begins or resumes Daily. It mixes the longest-overdue phrases and a random selection of
new phrases, up to the two daily limits. The defaults are 20 due and five new. Changing a limit in
**Settings** takes effect when you return to practice.

- Try saying the phrase, even when the card is new. Use **New cue**, **Hint**, or **More
  explanation** where available. **Reveal** shows the answer and reference audio.
- Rate what you said before looking at the answer or asking for help. If you could not recall
  the phrase, choose **Again**. The first rating starts its schedule. **Again**, **Hard**,
  **Good**, and **Easy** show when the phrase would come back, such as `3d` or `2.5m`.
  A phrase already graded today has no new interval to preview.
- Use **Skip** to continue without an assessment or practice credit. **Previous card** lets you
  revisit earlier revealed cards from this run. A skipped card can still be graded, and a graded card
  shows its saved rating and accepts another take. The right chevron moves back toward the
  current card. **More actions** lets you delete the displayed phrase after confirmation.

After a pass, **Extra** starts a fresh one when there is more eligible material beyond today's
allowance. A press extends each reached limit by one increment. For example, a limit of 20 due
reviews becomes 40 once you've completed 20. Pressing again without further practice gives no
additional allowance. A fresh pass can offer skipped cards again. Phrases scheduled for a future
date stay out of Daily and Extra. When no phrases are available, **Start** is disabled and
Practice can show the next due date.

## Record and compare

Tap the microphone to record, then tap it again to finish. Finishing a take on the front reveals
the answer and plays the reference followed by your recording. Rehearsing on the back uses the
same order. After you press **Ref** on that card, subsequent takes on its answer side play only
your recording automatically. Use **Mine** for your latest take or **Compare** to hear both.

Every completed take stays in the browser while it uploads in the background. You can rate or
move on while it uploads. Switching tabs finishes and keeps the take, and **Start** resumes the
regular card. A failed upload stays on the device for retry. The saved take includes the cue,
help, and answer shown during the attempt.

Choose **More actions → Recording history** on a revealed card to listen to earlier attempts.
The recordings are grouped by date, newest first. Tap a time to play that take. Tap it again
to stop, or tap another time to switch. **Show older recordings** loads earlier attempts.
**Back to practice** or Escape stops playback and returns you to the same card.

In Library phrase details, the history icon opens the saved takes inline. Choose **Mine** to listen.

## Find and practise one phrase

The **Library** lists phrases by lesson. Search across lessons by wording, meaning, situation, or
use. Open a phrase to see its meaning, lesson, source, reference audio, next due date, and saved
takes. Looking up a phrase leaves its schedule alone. If you are looking for new material, use
**Send “…” to Coach** or **Not it?** from the search results.

Choose **Practice** in phrase details to open the same hidden-answer card, then rate or skip to
return to the details. This practice is available outside the daily allowance. The first rating
of the phrase on a learning day can reschedule it even before its due date.

**Delete** removes the phrase, reference audio, and saved recordings after confirmation.
Practice history keeps the wording of completed phrases. A take still waiting to upload stays
on the device until you discard it. If deleting files fails, use **Retry cleanup**.

## Track practice and adjust settings

Below the Practice controls, **Today** shows completed recalls, new phrases, ratings, and
recordings, with pending takes counted separately. **See today’s phrases** includes wording
from cards you completed and later deleted. **Progress** adds up your practice days, ratings,
and recordings. **Total** shows how many phrases are due now, due later, or new. It also shows
a long-term memory count based on the scheduler's estimate of each phrase's stability.

A day with at least one rating counts toward the streak. The current streak still shows
yesterday's run until you practise today. Missing a full day resets the current streak. The
best streak and practice history stay visible.
Set a weekly target of one to six days in **Settings** and pause or change it any time. Each
Monday starts a fresh week. The target affects only the progress display.

**Settings** also holds your native language, learning timezone, daily limits, and return pace.
Pick **Often**, **Balanced**, or **Rarely**. The choices show how often a phrase rated **Good**
each time would come back in its first month and year.

## Use another device

On an iPhone, open your [private HTTPS address](private-access.md) in Safari and use
**Share → Add to Home Screen**. The icon opens Phrase Gym without the browser toolbar and
still needs the running server. Android browsers that support app shortcuts also offer
**Coach** from the icon. The pages follow your device's light or dark appearance.

The server runs on macOS and Linux. Use a macOS browser for the currently supported audio
playback and recording workflow. The server listens only on loopback by default and keeps
learner data in its local application-data directory.

See [backup and restore](backup-and-restore.md) for maintenance and the
[HTTP API reference](reference.md) for external agents.
