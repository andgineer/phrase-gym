import os
import sys
import tempfile
import threading
from pathlib import Path

import uvicorn

from phrase_gym.coach.main import run_coach


def main() -> None:
    port = int(sys.argv[1])
    with tempfile.TemporaryDirectory(prefix=f"phrase-gym-browser-{port}-") as directory:
        os.environ["PHRASE_GYM_DATA_DIR"] = str(Path(directory))
        os.environ["PHRASE_GYM_FAKE_TTS"] = "1"
        os.environ["PHRASE_GYM_FAKE_TTS_FAILURES"] = "1"
        os.environ["PHRASE_GYM_FAKE_TTS_FAIL_TEXT"] = "Let's get started."
        if os.environ.get("PHRASE_GYM_BROWSER_NO_COACH") != "1":
            threading.Thread(
                target=run_coach,
                args=(f"http://127.0.0.1:{port}",),
                daemon=True,
            ).start()
        uvicorn.run("phrase_gym.server.app:create_app", factory=True, host="127.0.0.1", port=port)


if __name__ == "__main__":
    main()
