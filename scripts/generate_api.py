import json
import os
import tempfile
from pathlib import Path

from phrase_gym.server.app import create_app


def main() -> None:
    # The contract never depends on, or creates, a learner data directory.
    with tempfile.TemporaryDirectory(prefix="phrase-gym-contract-") as data_dir:
        os.environ["PHRASE_GYM_DATA_DIR"] = data_dir
        document = json.dumps(create_app().openapi(), indent=2, sort_keys=True) + "\n"
    Path("frontend/openapi.json").write_text(document)


if __name__ == "__main__":
    main()
