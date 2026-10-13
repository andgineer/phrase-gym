from pathlib import Path

from fastapi import FastAPI, Request
from fastapi.responses import Response


def legacy_worker_fixture(app: FastAPI) -> None:
    app.state.legacy_worker = False
    source = (Path(__file__).parent / "fixtures/legacy-worker.js").read_text()
    source = source.replace("SHELL_VERSION", "legacy-test").replace(
        "SHELL_ASSETS",
        '["/index.html"]',
    )

    @app.post("/_test/legacy-worker/{enabled}", include_in_schema=False)
    def switch(enabled: bool):
        app.state.legacy_worker = enabled
        return {"enabled": enabled}

    @app.middleware("http")
    async def deliver(request: Request, call_next):
        if request.url.path == "/sw.js" and app.state.legacy_worker:
            return Response(source, media_type="application/javascript")
        return await call_next(request)
