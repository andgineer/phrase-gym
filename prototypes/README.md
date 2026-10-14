# Disposable browser probes

These standalone probes use browser-local test data. They are separate from the production application and its server-owned progress. Prototype-only behavior does not define product scope; use [current decisions](../specs/START-HERE.md) for that.

## Run locally

```sh
python3 prototypes/serve.py
```

Open `http://127.0.0.1:8871/learn/en` or `/audio-probe/`. Stop with Ctrl-C; choose another `--port` if needed. Settings, takes and imported references belong to that browser origin. Reset controls remove disposable test data. Nothing is uploaded to a provider or the production server.

The included English Samantha clips are feasibility assets, not production voice choices. Serbian reference audio needs an exact imported clip; there is no English fallback. Synthetic tones and local playback do not certify actual headset or speech quality.

## Phone access

An iPhone needs a reachable trusted HTTPS origin. Supply an already trusted certificate and private address, or use a private reverse proxy:

```sh
python3 prototypes/serve.py --host PRIVATE_IP --port 8872 --cert /path/to/cert.pem --key /path/to/key.pem
```

The server refuses non-loopback HTTP and prints an access link for its private cookie. Keep certificates and recordings outside Git.

## Checks

```sh
node --test prototypes/tests/core.test.mjs
cd prototypes
npm ci
npx playwright install chromium webkit
npm run test:browser
npm run test:review
npm run test:audio
```

Browser checks use synthetic capture and disposable profiles. Override `PHRASE_GYM_TEST_ORIGIN` for another port and `PHRASE_GYM_TEST_ARTIFACTS` for temporary output. Production checks are documented in [AGENTS.md](../AGENTS.md).
