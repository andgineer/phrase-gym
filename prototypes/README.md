# Phrase Gym P1 prototypes

Disposable interaction and device prototypes, built from the current specifications on 2026-10-12. They do not modify the Python application scaffold or implement production APIs, FSRS or server-side recording storage. P1's exit gate remains pending user review and real-device/content evidence. See [the evidence report](../specs/plan/evidence/p1-2026-10-12.md).

## Run locally

From the repository root:

```sh
python3 prototypes/serve.py
```

Open `http://127.0.0.1:8871/learn/en` for the drill or `/audio-probe/` for the device probe. No runtime packages or build step are required. The server serves only prototype assets; the original rich-click entry point remains unchanged. Stop with Ctrl-C. Choose a different `--port` if necessary.

Select your native language on first launch. English and Serbian learning URLs are bookmarkable; explanations support English, Russian and Serbian. The settings toggle makes controls follow the learning language. Native explanation text does not follow that toggle.

Data belongs to the browser origin: settings, per-language session checkpoints, exposure/result events, all audio takes and locally imported references are stored in IndexedDB. Sessions and actual captured audio survive ordinary reload/restart. Interrupted chunks remain incomplete; they do not become completed responses. Nothing is uploaded to an AI/provider/server. Browser eviction is not a backup; P3 supplies durable app-database storage. The reset/delete controls explicitly remove disposable test data; there is no automatic expiry or manual recording Save action.

The eight English WAVs are whole-phrase macOS Samantha speech prepared locally for feasibility checks. This is not a production voice selection. Serbian audio is deliberately missing until a matching clip is imported; there is no English fallback. Actor/course input has priority over generated assets. Import a checked whole-phrase clip with provenance in the probe, then replay it from either prototype on the same origin.

These files play in the browser on supported platforms; macOS is not required to run either prototype. The production preparation direction is echo-words’ Piper/Edge engines and llmbroker for LLM preparation, with provider secrets kept server-side in `.env`. No LLM call is needed for these manually authored fixtures. The script below only reproduces the old disposable Samantha assets; it is not the production provider. If those test clips need rebuilding on macOS with an installed Samantha voice and ffmpeg:

```sh
python3 prototypes/prepare_audio.py
```

Preparation refuses empty output. It does not generate speech while drilling or change another project's sources.

## Private HTTPS device use

Desktop loopback HTTP permits secure-context browser checks. An iPhone needs a **trusted HTTPS origin reachable from that phone**. A self-signed desktop certificate accepted only by a test harness is not evidence of phone trust or feasibility. No trust-store changes, public deployment or production hosting were performed.

The server supports a supplied development certificate/key and binds to a specific private address:

```sh
python3 prototypes/serve.py --host PRIVATE_IP --port 8872 --cert /path/to/trusted-dev-cert.pem --key /path/to/dev-key.pem
```

Replace the address and paths with a private interface and a certificate already trusted by the phone and valid for that address/name. The server refuses non-loopback HTTP. It prints a random access link; open it on the phone to set the private access cookie. Unauthorized network requests receive 403. No third-party tunnel or public listener is configured. A private HTTPS reverse proxy can instead reach the loopback listener with its own owner-device access policy.

The app has a manifest and service worker. On Safari, use Add to Home Screen and repeat the probe in that mode. Download prepared assets while connected before disconnecting. The cache holds the shell and explicitly downloaded reference clips, not arbitrary future sessions. Only prepared, downloaded work is claimed by the prototype. Reload once if a newly installed service worker has not taken control yet. After updating the prototype, reload and download again to cache the reviewed shell and assets.

Use the probe's result selectors for observed results. Untested cases stay untested; export the metadata-only report. Do not put private recordings in evidence or source control. Temporary certificate keys also stay outside the repository.

## Current manual-feedback prototype

Version **0.1.5** follows the [unified practice-card decision](../specs/decision-unified-practice-card.md). Native/learning UI switching is available at first launch and in Settings. One card always offers recording, replay, reference and red/amber/green self-check buttons. There is no recording switch, Practice again, problem submenu or supported-only Continue button.

Show answer fills the gaps. The adjacent blue book-icon Show & explain action expands meaning, usage and phrase breakdown within the same card. Recording is optional; use Show answer and rate without accessing the microphone. Supported ratings retain their support metadata rather than being counted as independent recall.

The round microphone becomes a stop control while recording or playing audio. Stopping playback restores the microphone. After the first explicit recording request, the microphone connection remains ready between takes to avoid repeated headset/audio-route initialization. Only explicit takes are recorded. Pause, leaving practice, backgrounding or finishing releases the microphone. Show **Press again to stop recording** only during active capture; remove it as soon as capture stops. There is no idle microphone-ready label. The Undo banner says **Previous rating saved**, referring to the rating for the preceding phrase.

## Walkthroughs

1. Choose the native language and optional learning-language UI at first launch. Switch later under Settings. Explanations stay native.
2. Start a session. Without recording, press Show answer and rate your recall using the colored/icon buttons. Direct rating is also available; no microphone permission is required.
3. Open Show & explain. It expands inside the card beside Show answer, keeps the full phrase visible and retains all practice controls. Rating remains available; support is stored separately. Collapse it using the same link.
4. Record, stop, and listen to your take followed by the reference. The full phrase is shown after capture. Interrupt playback using the round stop icon; it returns to a microphone. Record again directly. Every take remains stored; the first response's evidence is preserved. Grade to advance, and use Undo to correct the result.
5. Search lessons for `rap` and phrases for `greetings`. Browsing a phrase opens its materials without starting a review. Visit its lesson list and start a drill. Recent visible answers are tracked as support, while self-check controls remain available.
6. Pause, reload, resume and switch learning languages. Sessions and recordings are scoped to the language URL. Pause/leave/background releases the ready microphone. English and Serbian bookmarks remain independent.
7. Try a prepared dialogue, then isolate a turn and rate to advance within the exchange. One target identity is shared across ordinary drills and dialogue appearances.
8. Mark Not yet, advance the diagnostic clock five minutes and complete two other targets to exercise a separated retry. This is a policy probe, not production FSRS.
9. Download the prepared English session and reload offline. Reference playback, self-check and retained takes remain usable. Serbian reference audio is still pending.
10. On Safari with the actual headset, try several takes of a continuous sentence. Listen for the reported mid-take interruption, including the first take, later takes and resuming after Pause. Synthetic checks do not establish that this device-specific issue is resolved.

## Checks

Node 20+ runs the policy/fixture checks with no dependencies:

```sh
node --test prototypes/tests/core.test.mjs
```

Install the prototype's pinned development browser dependency separately from the Python scaffold, then run the localhost server and tests:

```sh
cd prototypes
npm install
npx playwright install chromium webkit
npm run test:browser
npm run test:review
npm run test:audio
```

The browser runner uses port 8871 by default; override `PHRASE_GYM_TEST_ORIGIN` if needed. It writes metadata/screenshots to `/tmp/phrase-gym-p1-browser` (or `PHRASE_GYM_TEST_ARTIFACTS`). It uses a synthetic microphone input, actual MediaRecorder/IndexedDB/decoding, 320px layout, 200% text and injected quota faults. It is **not** actual iPhone, microphone, headset or actor-audio evidence.

Fresh-session review corrections and verification are tracked in [the review-loop report](../specs/plan/evidence/p1-review-loop-2026-10-12.md). The focused browser suite covers browsing exposure, cross-tab history, Undo/retries, original recording replay, unsaved-take recovery, cancellation during delayed audio loading and actor-audio priority.

The audio-continuity check uses a generated tone and disposable persistent browser profiles in Chromium and WebKit. It verifies three stored takes, one microphone-stream acquisition, release on cancel and no middle-of-take dropouts in the encoded signal. It does not access the real headset or prove its clipping is fixed.
