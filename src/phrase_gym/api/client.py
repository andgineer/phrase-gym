from pathlib import Path

from fastapi import FastAPI, Request
from fastapi.responses import FileResponse, JSONResponse
from starlette.staticfiles import StaticFiles


def mount_client(app: FastAPI, client_dir: Path | None) -> None:
    @app.middleware("http")
    async def connected_responses(request: Request, call_next):
        response = await call_next(request)
        response.headers["Cache-Control"] = "no-store"
        return response

    if client_dir and (client_dir / "index.html").is_file():
        assets = client_dir / "assets"
        if assets.is_dir():
            app.mount("/assets", StaticFiles(directory=assets), name="assets")

        @app.get("/sw.js", include_in_schema=False)
        def retire_worker():
            return FileResponse(
                Path(client_dir or ".") / "retire-worker.js",
                headers={"Cache-Control": "no-store"},
            )

        @app.get("/icon.svg", include_in_schema=False)
        def icon():
            return FileResponse(
                Path(client_dir or ".") / "icon.svg",
            )

        @app.get("/", include_in_schema=False)
        @app.get("/index.html", include_in_schema=False)
        @app.get("/learn", include_in_schema=False)
        @app.get("/learn/{client_path:path}", include_in_schema=False)
        def client():
            return FileResponse(
                Path(client_dir or ".") / "index.html",
                headers={"Cache-Control": "no-store"},
            )
    else:

        @app.get("/", include_in_schema=False)
        def no_client():
            return JSONResponse(
                {"detail": "Build the client and pass --client-dir frontend/dist"},
                503,
            )
