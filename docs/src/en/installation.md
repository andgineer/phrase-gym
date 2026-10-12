# Run the current source build

Use Python 3.12+, uv and Node 22.12+. From the project checkout:

```bash
uv sync --frozen
cd frontend
npm ci
npm run build
cd ..
uv run phrase-gym --data-dir .data serve --client-dir frontend/dist
```

Open `http://127.0.0.1:8000/learn/en`. The private `.data` directory contains the SQLite database and original reference media. Keep it between restarts. Without `--data-dir`, the app uses the operating system's user application-data directory.

The collection begins empty. Validate and stage a reviewed manual JSON package with `phrase-gym import-package PACKAGE --stage`, inspect its returned report, then run `phrase-gym publish IMPORT_ID --reviewed`. The project includes English and Serbian structural examples in `fixtures/`.

`./build.sh` builds a wheel with the client bundled. Installing that wheel locally supplies the same `phrase-gym serve` command without a separate client directory. The current source foundation is not a claim that the published PyPI package contains this application.

This local server binds `127.0.0.1`. Private HTTPS hosting and real-phone checks follow in later phases.
