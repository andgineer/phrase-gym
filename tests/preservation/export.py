"""Rewrite the scheduler preservation fixture from the locked Python implementation.

Run `uv run python -m tests.preservation.export`, then review the diff before committing.
"""

import json

from tests.integration.helpers import ROOT
from tests.preservation import fsrs_cases

FSRS_FIXTURE = ROOT / "fixtures/preservation/fsrs/review-policy.json"


def fsrs_text() -> str:
    return json.dumps(fsrs_cases.fixture(), indent=2) + "\n"


def main() -> None:
    FSRS_FIXTURE.write_text(fsrs_text(), encoding="utf-8")


if __name__ == "__main__":
    main()
