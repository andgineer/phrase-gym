# Add your phrases

You can add phrases and words from your courses, notes or other learning material without
filling in individual phrase cards or handling any file.

## Type a phrase or word for Coach

Open the **Coach** tab and type a phrase or word, optionally followed by the sentence it came
from. Press **Send** or Enter. Pasted line breaks become spaces. For example, try
`take it with a grain of salt — I'd take his estimate with a grain of salt`. The sentence helps
Coach choose the right meaning; without it, Coach uses the most common useful one.

Phrase Gym saves the text at once, even if [Coach is offline](installation.md#preparation-with-coach).
It clears the field and briefly confirms. Your practice continues meanwhile. Coach later prepares a complete
card: a phrase becomes a card for its reusable chunk, and a single word becomes a card for one of
its meanings with two or three example contexts. Each card gets reference audio and appears in
the **Added with Coach** lesson, apart from your course lessons. Phrase Gym does not show
preparation progress.

Typing the same text again never adds a second card. The field tells you when the text is
**Already waiting for Coach** or **Already in your Library**. Text whose card you deleted is
prepared again.

When Coach fixes an obvious typo, such as **definately → definitely**, the **Coach** page lists
the change and the card says what it corrected. A valid variant, such as **colour**, keeps its form.

If Coach cannot use the text, for example because it is not English, cannot be read, could be
several different words or is too long for one card, the **Coach** page lists the text, its
learning language and the reason. With Gemini free the reason is in your native language; with
Claude or Codex it is still in English.
Close it with **Dismiss**. Typing the same text again shows that result instead of preparing it
again, so change the text to try something else.

If a service problem stops Coach, for example because the CLI is not logged in or ran out of
quota, Coach prints the reason in its terminal, the **Coach** page offers **Retry**, and the
Practice page shows a notice below **Total**. Close the notice with **×** once you have read it.
Retrying from Coach or typing the same text again prepares it again.

## Ask your personal AI agent

Pass [the agent instructions](https://andgineer.github.io/phrase-gym/agent-add-phrases/) to your AI agent; you do not need to read them.
Tell it where to find your material and give it your Phrase Gym server address. It prepares the
text, reuses suitable source recordings or generates missing audio, and submits the complete
material directly to Phrase Gym. Once the server confirms the import, the phrases are immediately
available for learning. Ask the agent to report what was added and any material it could not
finish.

The agent must be able to reach your server and prepare audio. If either is unavailable, resolve
that prerequisite with the agent before continuing.
