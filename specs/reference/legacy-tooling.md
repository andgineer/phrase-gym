# Legacy phrase source format and tooling

Reference snapshot from the original phrase-shadowing trainer, retained here so the next session can understand source intake without the old workspace. These publishing/course instructions describe historical tooling; they are not authorization to run it. The old app's cue rules and progress store are not the Phrase Gym design.

The corresponding source files are archived in `archive/phrase-shadowing-before-move.zip`. Current Phrase Gym intake architecture is in [data contracts](../architecture/data-contracts.md).

---

# phrase-shadowing

Shadowing trainer "Английское эхо" for course phrases recorded by voice actors, plus phrases voiced by neural TTS.

- Published page: https://claude.ai/artifact/Jwe4YpVMthy8UjerkdEKY2 (progress lives in the artifact's `db`, per user)
- Source of phrases: lesson notes in the Obsidian vault, `~/obsidian/kb/languages/english/фразы/`, audio in its `_assets/`
- `index.html` — the page template; `phrases.py build` injects the lesson data into it

## Lesson note format

Same as `переговоры 1.2 building rapport.md`:

- front matter: `id:` (lesson id, also the audio prefix), `курс:`, `урок:`, optional `источник:`, `порядок:` (lessons are ordered by it; course lessons use module+lesson, e.g. `12`; extras `90`), optional `голос a:` / `голос b:` (edge-tts voices for the dialog sides; default `en-GB-RyanNeural` / `en-GB-SoniaNeural`)
- `## фразы` → `### <topic>` → numbered items: text line, indented `![[audio]]`, indented `*English situation cue*` (no Russian — the learner is B2 and translations distract)
- `## диалоги` → `### <English title>`, lines `- A: text ![[audio]]` (the page describes a dialog by the cues of the phrases in it, so no scene text is needed); `**bold**` marks the phrase inside a longer line; `<!-- voice: en-GB-RyanNeural -->` on a line overrides its TTS voice
- other `##` sections are ignored by the build

Audio names decide how a dialog line links to a phrase: `<id>-NN` is phrase NN, `<id>-NNa`/`NNb` is another recording of phrase NN (a part of it, or the whole dialog line around it), `<id>-dK-M` is a dialog line that is not a phrase. A `-tts` suffix marks synthetic audio; the page labels it "ИИ-голос".

## Adding a course lesson

1. Collect phrase texts and audio URLs from the course page. On elearn.epam.com the unit content is in `#unit-iframe` (same origin); each `<tr>` holds the text and an `<audio><source src>`. Course assets under `/assets/courseware/` download without auth, but only with curl (Python's urllib gets 403). Lessons the user is not enrolled in return empty content; enrolling needs the user's explicit permission.
2. Write `urls.txt` with lines `<number> <audio url>` and run
   `python3 phrases.py fetch <lesson-id> urls.txt`
   It cuts the spoken number ("One.", "Two." …) off each clip and writes `_assets/<lesson-id>-NN.mp3`. A clip where no number is detected is copied untrimmed and reported — check it by ear.
3. For a two-voice exchange that dialogs need as separate lines: `python3 phrases.py split _assets/<file>.mp3` → `<file>a.mp3`, `<file>b.mp3`.
4. Write the lesson note. Dialog lines with no actor recording get an embed `![[<id>-dK-M-tts.mp3]]`; pick a voice of the opposite sex to the actor of the neighbouring line (the course has one male and one female actor).

## Phrases without recordings

Write the note with `-tts` embeds for every phrase and dialog line, then `python3 phrases.py tts`. It synthesizes every missing `*-tts.mp3` with edge-tts (run through `uvx`, needs network), trims silence and levels it to the actors' loudness (−20 LUFS). A phrase clip uses the voice of the dialog side that says it. `--force` re-synthesizes existing clips.

## Publishing

`python3 phrases.py build <out-dir>` (use the session scratchpad), then republish `<out-dir>/index.html` to the artifact URL above with the new `audio/*.mp3` files in `files` (files already published are kept). An artifact holds at most 255 files; at about 80 now, merging clips per lesson will be needed after roughly ten more lessons.
