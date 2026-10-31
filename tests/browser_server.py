import os
import sys
import tempfile
import threading
from datetime import UTC, datetime
from pathlib import Path

import uvicorn

from phrase_gym.coach.main import run_coach
from phrase_gym.server.app import create_app

CLOCKED_SERVICES = (
    "current",
    "daily",
    "extra",
    "phrase_practice",
    "view_actions",
    "completion",
    "lessons",
    "facts",
)


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
        app = create_app()
        simulated_time: list[datetime | None] = [None]
        for service in CLOCKED_SERVICES:
            getattr(app.state, service).clock = lambda: simulated_time[0] or datetime.now(UTC)

        @app.post("/__test/clock/{iso_time}")
        def set_clock(iso_time: str) -> dict[str, str]:
            simulated_time[0] = datetime.fromisoformat(iso_time)
            return {"time": simulated_time[0].isoformat()}

        test_route = app.router.routes.pop()
        app.router.routes.insert(-1, test_route)

        uvicorn.run(app, host="127.0.0.1", port=port)


if __name__ == "__main__":
    main()
