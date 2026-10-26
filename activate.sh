#!/usr/bin/env bash

if ! (return 0 2>/dev/null); then
    echo 'Source this script: . ./activate.sh' >&2
    exit 1
fi

if ! command -v uv >/dev/null; then
    echo 'Install uv first: https://docs.astral.sh/uv/getting-started/installation/' >&2
    return 1
fi

uv sync --frozen --all-extras || return 1
. .venv/bin/activate
