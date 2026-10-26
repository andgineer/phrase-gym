import os
import sys
import tempfile
from pathlib import Path

import uvicorn


def main() -> None:
    port = int(sys.argv[1])
    with tempfile.TemporaryDirectory(prefix=f"phrase-gym-browser-{port}-") as directory:
        os.environ["PHRASE_GYM_DATA_DIR"] = str(Path(directory))
        os.environ["PHRASE_GYM_FAKE_TTS"] = "1"
        os.environ["PHRASE_GYM_FAKE_TTS_FAILURES"] = "1"
        os.environ["PHRASE_GYM_FAKE_TTS_FAIL_TEXT"] = "Let's get started."
        uvicorn.run("phrase_gym.app:create_app", factory=True, host="127.0.0.1", port=port)


if __name__ == "__main__":
    main()
