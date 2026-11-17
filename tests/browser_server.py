import asyncio
import os
import random
import sys
import tempfile
from datetime import UTC, datetime
from pathlib import Path

import uvicorn
from starlette.applications import Starlette
from starlette.requests import Request
from starlette.responses import JSONResponse
from starlette.routing import Mount, Route
from starlette.types import Receive, Scope, Send

from phrase_gym.coach.claude_cli import ClaudeCli
from phrase_gym.coach.main import CoachApi, process_once
from phrase_gym.coach.preparation import CardPreparer
from phrase_gym.coach.speech import PiperSpeech
from phrase_gym.server.app import create_app
from tests import fake_cli

CLOCKED_SERVICES = (
    "current",
    "today",
    "daily",
    "extra",
    "phrase_practice",
    "view_actions",
    "completion",
    "lessons",
    "facts",
)


class BrowserServer:
    def __init__(self, directory: Path, url: str):
        self.directory = directory
        self.api = CoachApi(url)
        self.coach_task: asyncio.Task | None = None
        self.wake = asyncio.Event()
        self.generation = 0
        self.simulated_time: datetime | None = None
        self.coach_enabled = False
        self.new_app()
        self.control = Starlette(
            routes=[
                Route("/__test/reset", self.reset, methods=["POST"]),
                Route("/__test/clock/{iso_time}", self.set_clock, methods=["POST"]),
                Mount("/", app=self.dispatch),
            ]
        )

    def new_app(self) -> None:
        self.generation += 1
        directory = self.directory / str(self.generation)
        directory.mkdir()
        os.environ["PHRASE_GYM_DATA_DIR"] = str(directory)
        self.app = create_app()
        self.simulated_time = None
        rng = random.Random(7)
        for name in CLOCKED_SERVICES:
            service = getattr(self.app.state, name)
            service.clock = lambda: self.simulated_time or datetime.now(UTC)
            if hasattr(service, "rng"):
                service.rng = rng
        cli = ClaudeCli(
            str(fake_cli.executable(directory, "claude")), attempts=directory / "attempts"
        )
        self.preparer = CardPreparer(cli, cli.attempts)
        self.speech = PiperSpeech(None, fake=True)

    async def reset(self, request: Request) -> JSONResponse:
        # A preparation must finish against its original app before the next test replaces it.
        if self.coach_task is not None:
            await self.coach_task
        self.wake.clear()
        self.new_app()
        self.coach_enabled = (await request.json())["coach_enabled"]
        return JSONResponse({"generation": self.generation})

    async def set_clock(self, request: Request) -> JSONResponse:
        self.simulated_time = datetime.fromisoformat(request.path_params["iso_time"])
        return JSONResponse({"time": self.simulated_time.isoformat()})

    async def drain_coach(self) -> None:
        while self.wake.is_set():
            self.wake.clear()
            while await asyncio.to_thread(process_once, self.api, self.preparer, self.speech):
                pass

    async def dispatch(self, scope: Scope, receive: Receive, send: Send) -> None:
        await self.app(scope, receive, send)
        if (
            self.coach_enabled
            and scope["method"] == "POST"
            and scope["path"] == "/api/coach/requests"
        ):
            self.wake.set()
            if self.coach_task is None or self.coach_task.done():
                self.coach_task = asyncio.create_task(self.drain_coach())


def main() -> None:
    port = int(sys.argv[1])
    with tempfile.TemporaryDirectory(prefix=f"phrase-gym-browser-{port}-") as directory:
        server = BrowserServer(Path(directory), f"http://127.0.0.1:{port}")
        uvicorn.run(server.control, host="127.0.0.1", port=port)


if __name__ == "__main__":
    main()
