#!/usr/bin/env bash
# Build the same-origin client into the Python wheel and source distribution.
set -euo pipefail
cd "$(dirname "$0")"
uv sync --frozen
(
  cd frontend
  npm ci
  npm run typecheck
  npm run build
)
uv run python -c 'from pathlib import Path; import shutil; shutil.rmtree("src/phrase_gym/static", ignore_errors=True); shutil.copytree("frontend/dist", "src/phrase_gym/static")'
uv build
