import json
from pathlib import Path

from phrase_gym.app import create_app


def main() -> None:
    document = json.dumps(create_app().openapi(), indent=2, sort_keys=True) + "\n"
    Path("frontend/openapi.json").write_text(document)


if __name__ == "__main__":
    main()
