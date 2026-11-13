# Add your phrases

You can add phrases and words from your courses, notes or other learning material without
filling in individual phrase cards or handling any file.

## Type a phrase or word for Coach

Type one phrase or word into the field at the top right and press **Coach**. You can add the
sentence it came from after it, for example
`take it with a grain of salt — I'd take his estimate with a grain of salt`. The sentence helps
Coach choose the right meaning; without it, Coach uses the most common useful one.

Phrase Gym saves the text at once, whether or not [Coach is running](installation.md), clears the
field and briefly confirms. Your practice continues meanwhile. Coach later prepares a complete
card: a phrase becomes a card for its reusable chunk, and a single word becomes a card for one of
its meanings with two or three example contexts. Each card gets reference audio and appears in
the **Added with Coach** lesson, apart from your course lessons. Phrase Gym does not show
preparation progress.

Typing the same text again never adds a second card. The field tells you when the text is
**Already waiting for Coach** or **Already in your Library**. Text whose card you deleted is
prepared again.

If Coach cannot prepare a card, for example because the text contains no English phrase or word
or because the Claude CLI is not logged in, it prints the reason in its terminal and the Practice page
shows a notice below **Total**. Close it with **×** once you have read it. To retry, type the
same text into the field again.

## Ask your personal AI agent

Pass [the agent instructions](https://andgineer.github.io/phrase-gym/agent-add-phrases/) to your AI agent; you do not need to read them.
Tell it where to find your material and give it your Phrase Gym server address. It prepares the
text, reuses suitable source recordings or generates missing audio, and submits the complete
material directly to Phrase Gym. Once the server confirms the import, the phrases are immediately
available for learning. Ask the agent to report what was added and any material it could not
finish.

The agent must be able to reach your server and prepare audio. If either is unavailable, resolve
that prerequisite with the agent before continuing.
