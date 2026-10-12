# Current source build

Use Python 3.12+, uv and Node 22.12+. From the project checkout:

```bash
uv sync --frozen
cd frontend
npm ci
npm run build
cd ..
uv run phrase-gym --data-dir .data serve --client-dir frontend/dist
```

Open `http://127.0.0.1:8000/learn/en` and choose your native language. Persist the `.data` directory between restarts. The current foundation is available from source; its implementation does not imply a PyPI release.

See `specs/plan/p2-runbook.md` in the checkout for content import, reference media, checks and packaging.
