# Add your phrases

You can add phrases and words from your courses, notes or other learning material without
filling in individual phrase cards or handling any file.

## Type a phrase or word for Coach

Open the **Coach** tab and type a phrase or word, optionally followed by the sentence it came
from. Coach reads it in the language whose pages you are on, such as German at `#de/coach`. Press **Send** or Enter. Pasted line breaks become spaces. For example, try
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

If Coach cannot use the text, for example because it is not in the language you are learning on
that page, cannot be read, could be several different words or is too long for one card, the
**Coach** page lists the text, its learning language and the reason in your native language.
Close it with **Dismiss**. Typing the same text again shows that result instead of preparing it
again, so change the text to try something else.

If a service problem stops Coach, for example because the CLI is not logged in or ran out of
quota, Coach prints the reason in its terminal, the **Coach** page offers **Retry**, and the
Practice page shows a notice below **Total**. Close the notice with **×** once you have read it.
Retrying from Coach or typing the same text again prepares it again.

German cards follow your German choices in **Settings**. With **Translation in native language**
on, which is where German starts, a card asks in your native language for the German word or
phrase: its cue gives the meaning and its short help explains it in your native language, while
the answer, the example sentence and the reference audio stay German. A noun is practised with its article, such as
**die Tür**, and the help mentions gender and plural where useful. Capital letters count:
**essen** (to eat) and **das Essen** (food) are different words, and Coach keeps the one you typed.
English cards keep their English cues and short help; the longer explanation behind **More
explanation** is in your native language for every language. The German reference voice is set
up as described in the [voice setup guide](piper-voice.md); without it, German text waits in the
**Coach** page with **Retry** and the reason that Coach has no voice for the language yet.

## Ask your personal AI agent

Pass [the agent instructions](https://andgineer.github.io/phrase-gym/agent-add-phrases/) to your AI agent; you do not need to read them.
Tell it where to find your material and give it your Phrase Gym server address. It prepares the
text, reuses suitable source recordings or generates missing audio, and submits the complete
material directly to Phrase Gym. Once the server confirms the import, the phrases are immediately
available for learning. Ask the agent to report what was added and any material it could not
finish.

The agent must be able to reach your server and prepare audio. If either is unavailable, resolve
that prerequisite with the agent before continuing.
