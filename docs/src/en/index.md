# Phrase Gym

Phrase Gym trains selected words in useful phrases, fixed expressions and prepared dialogues toward ready use.

The current source build provides a library, native-language explanations, language bookmarks, concept lookup and imported reference audio. Guided sessions, optional learner recording, prepared dialogue rehearsal and FSRS spaced repetition are implemented in the local core trainer.

Choose your native language on first launch. The interface can use that language or follow your learning language. Explanations always remain in your native language. Bookmark `/learn/en` or `/learn/sr` to keep language views separate.

Use **Find a phrase** to search wording or prepared concepts such as `greetings`. Opening a result shows its meaning and usage without grading it. Lesson links open the complete phrase list. Available **Reference** audio plays when requested; missing audio is shown explicitly. Start session uses due/new phrases from All lessons or the selected lesson. Try the whole phrase, record optionally or Show answer, then choose Not yet, With effort (correct recall with difficulty), or Recalled. Show & explain opens native-language help inline. Every take is retained; pending/error states offer Retry. Resume preserves the session's original language. Actual phone/offline and content acceptance remain later work.

See [installation](installation.md) to run the source build. For all local tools, run:

```bash
phrase-gym --help
```
