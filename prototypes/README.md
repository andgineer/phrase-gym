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

## Accepted manual feedback, 2026-10-12

Choose native-language UI or **Use learning language for the interface** at first launch; change it later under **⚙ Settings**. Explanations retain the native language. Fresh sessions default to recording on; capture starts only when you press the microphone. Existing sessions retain their selected mode on resume. **Show answer** reveals the full phrase without an explanation and records supported exposure. **Other answer / problem** has been removed entirely. See [the decision](../specs/decision-p1-manual-feedback.md).

## Walkthroughs

1. Search lesson titles for `rap`, then clear/select All lessons. Search phrases for `greetings`; the matching wording does not contain that concept. Inspect an explanation, return with the query intact, visit its complete lesson list and start a lesson drill. Browsing creates exposure, never a recall result.
2. Turn recording off and start a familiar-item session directly. Reveal after silent retrieval and choose Not yet, With effort or Recalled. What counts? gives the original-attempt criteria. Silent work is memory evidence only.
3. On a cue, open Show & explain before responding, then return. The drill hides native-language meaning but retains support. Reveal now offers Continue without a recall grade. A separate I couldn’t recall it action can record one reported failure before help.
4. Request Practice again on the answer card. First retain the original result; extra practice shows the complete phrase and cannot overwrite that result. Undo corrects the original event while retaining takes/history. Allowed equivalents count as correct under What counts?
5. Leave recording on (or enable it after the silent walkthrough). Start with the microphone icon, stop manually, listen to the learner take with the cue retained, then see/hear the complete reference. Replay either version. Empty/failed capture creates no memory grade. Missing/blocked audio offers manual recovery.
6. Open Show & explain with recording on. The full phrase stays visible during capture and both playback stages. Its take is supported practice. Turn recording off to use explanation and manually requested reference audio without microphone access.
7. Open a prepared dialogue from Home. Partner/supporting lines surround target turns. Practise this turn isolates difficulty; retain its original result, practise, and continue into the next partner turn. There is one target identity across the dialogue and ordinary drill.
8. Pause, reload and resume. Switch to Serbian and back; the English session/history remain. Open English and Serbian in separate tabs; URL languages remain independent. Explicit session takeover pauses the old view (a prototype lease, not a production concurrency guarantee).
9. Inspect a word-retrieval target (`en-look`), a shown-construction use target (`en-use`, available under Practice all), and the frame's allowed inflection. The latter use task receives no word-recall credit. Rejected/unknown/ambiguous candidates stay in [the review sheet](drill-flow/cue-review.md), outside the practice queue.
10. For the retry engineering walkthrough, mark the first target Not yet, use **Prototype diagnostics → Advance probe clock +5 min**, and finish two other distinct targets. The failed target returns using another stored cue when suitable. The original failure remains; at most two extra retrieval attempts are queued. No timer forces waiting. Short sessions defer repairs. A fresh session with Practice all can demonstrate stored-mask variation; a single-cue target reuses its reliable cue.

The diagnostic clock is an engineering control, not a learner experiment or calibration task. Fixture due flags/completion receipts demonstrate selection, not FSRS dates. Failed targets remain due on later sessions; early voluntary practice does not promote a schedule. A production policy/FSRS adapter belongs to P3.

## Checks

Node 20+ runs the policy/fixture checks with no dependencies:

```sh
node --test prototypes/tests/core.test.mjs
```

Install the prototype's pinned development browser dependency separately from the Python scaffold, then run the localhost server and tests:

```sh
cd prototypes
npm install
npx playwright install chromium
npm run test:browser
npm run test:review
```

The browser runner uses port 8871 by default; override `PHRASE_GYM_TEST_ORIGIN` if needed. It writes metadata/screenshots to `/tmp/phrase-gym-p1-browser` (or `PHRASE_GYM_TEST_ARTIFACTS`). It uses a synthetic microphone input, actual MediaRecorder/IndexedDB/decoding, 320px layout, 200% text and injected quota faults. It is **not** actual iPhone, microphone, headset or actor-audio evidence.

Fresh-session review corrections and verification are tracked in [the review-loop report](../specs/plan/evidence/p1-review-loop-2026-10-12.md). The focused browser suite covers browsing exposure, cross-tab history, Undo/retries, original recording replay, unsaved-take recovery, cancellation during delayed audio loading and actor-audio priority.
