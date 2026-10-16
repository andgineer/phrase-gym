"""Rewrite preservation fixtures from the locked Python implementation.

Run `uv run python -m tests.preservation.export`, then review the diff before committing.
"""

import json

from tests.integration.helpers import ROOT
from tests.preservation import fsrs_cases, schema_dumps

FIXTURES = ROOT / "fixtures/preservation"
FSRS_FIXTURE = FIXTURES / "fsrs/review-policy.json"
SCHEMA_FIXTURES = FIXTURES / "schema"


def fsrs_text() -> str:
    return json.dumps(fsrs_cases.fixture(), indent=2) + "\n"


def schema_path(revision: str):
    return SCHEMA_FIXTURES / f"alembic-{revision}.sql"


def main() -> None:
    FSRS_FIXTURE.write_text(fsrs_text(), encoding="utf-8")
    for revision in schema_dumps.revisions():
        schema_path(revision).write_text(schema_dumps.dump(revision), encoding="utf-8")


if __name__ == "__main__":
    main()
