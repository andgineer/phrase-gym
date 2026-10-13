# P3 core trainer — run and verify

Use the same local process/data directory as the [P2 runbook](p2-runbook.md). No deployment, provider account or external delivery is required for reviewed manual content.

```bash
uv sync --frozen
npm --prefix frontend ci
npm --prefix frontend run build
uv run phrase-gym --data-dir .data serve --client-dir frontend/dist
```

Open `http://127.0.0.1:8000/learn/en`. Native-language Settings and learning-language bookmarks retain their existing behavior. Import reviewed packages and exact reference media using the P2 commands/API. The bundled example packages contain structural examples, not production speech assets; targets with missing reference/cue media remain Preparing. They are browseable/recordable as visible-model practice but cannot enter a ready retrieval session.

Start session uses All lessons or the selected lesson's due/new targets and starts immediately. If another session is open, that click replaces it while retaining its history; no second replacement action is required. Resume session preserves the pinned language and position. Changing the viewed language or returning to the library does not replace a session. Prepared dialogues appear separately in the library and share target schedules with ordinary drills.

Retrieval cards pair the situation with prepared, phrase-specific visible chunks and gaps. A broad situation alone is not a valid exact-phrase cue. Imports reject cues with no visible phrase text; content without a reliable cue remains Preparing. Imported cue revisions apply to newly started sessions, while earlier pinned manifests remain historical evidence.

On the practice card, attempt the whole phrase with its authored cue. Recording is optional. Show answer preserves the original self-report; Show & explain opens native-language help inline. Reference or explanation before responding remains support. Choose Not yet, With effort (correct recall with difficulty), or Recalled. Ratings advance quietly; With effort is not partial recall. The microphone captures another take directly, retaining the original response and every earlier recording. My recording uses the latest take.

The first capture keeps the gaps until it completes. An unfinished/recovered incomplete capture does not promote its schedule. A take ends with manual Stop (or an explicit interruption warning at the capture storage boundary), is journaled locally and retained as a database BLOB. Pending/error states offer Retry; one rejected take retains its bytes and error without blocking independent valid uploads. No manual Save or expiry exists. Pause/navigation/Settings/background release capture/playback. Reload reconciles the server checkpoint and pending events/takes with stable IDs. Review and take acknowledgements are distinct; an event save does not claim its audio reached the database.

The API/OpenAPI documentation at `/docs` describes session start/read/pause/resume/readiness, ordered evidence batches and receipts, and take upload/list/playback. Byte ranges are supported for retained recording playback. Do not put private recordings into fixtures, logs or source control. `.data` stays ignored; browser data is a pending journal/cache, not a backup.

Verify with `uv run inv pre` and then `uv run inv test`. Browser capture tests replace the microphone boundary with a generated oscillator and use the real MediaRecorder, IndexedDB, API, decoder and SQLite. They use disposable persistent profiles and explicitly override `navigator.mediaDevices` so WebKit cannot reach the operator's native microphone. Native test playback uses zero output volume so synthetic tones are inaudible while real playback events continue. The complete browser CI job uses macOS because Linux Playwright WebKit lacks MediaRecorder. Synthetic tests do not establish actual headset/iPhone or speech/content acceptance.

The [core decision](../decision-core-trainer.md) records budget, FSRS, support, repair and recovery defaults. Follow the remaining P4/P5/P6 work and [open problems](../open-problems.md), rather than restarting P1/P2 review loops or the stopped clipping investigation.
