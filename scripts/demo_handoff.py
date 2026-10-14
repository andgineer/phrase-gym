"""Build a portable public selected-sense export; delivery remains an explicit action."""

import argparse
import json
from pathlib import Path
from uuid import UUID

from phrase_gym.contracts.handoff import HandoffEnvelope
from phrase_gym.contracts.preparation import SourcePackage

EXPORT_ID = UUID("9bc9cbea-877d-4c36-bf8c-ce6ca45fc1fe")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    packet = json.loads(Path("tests/fixtures/preparation-rotation.json").read_text())
    source = SourcePackage.model_validate(packet["source"])
    envelope = HandoffEnvelope(
        export_id=EXPORT_ID,
        sender="echo-words-manual-contract-demo",
        selected_sense_id="greeting",
        selected_sense="Reply to a greeting and reciprocate it",
        package=source,
    )
    args.output.write_text(envelope.model_dump_json(indent=2) + "\n")
    print("Public envelope written; stable export ID:", EXPORT_ID)


if __name__ == "__main__":
    main()
