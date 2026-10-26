from urllib.parse import urlsplit

from starlette.responses import JSONResponse
from starlette.types import ASGIApp, Message, Receive, Scope, Send

MAX_REQUEST_BYTES = 16 * 1024 * 1024


class LocalBoundary:
    def __init__(self, app: ASGIApp) -> None:
        self.app = app

    async def __call__(self, scope: Scope, receive: Receive, send: Send) -> None:
        if scope["type"] != "http":
            await self.app(scope, receive, send)
            return
        headers = dict(scope["headers"])
        origin = headers.get(b"origin", b"").decode()
        host = headers.get(b"host", b"").decode()
        same_origin = bool(origin) and urlsplit(origin).netloc == host
        if scope["method"] not in {"GET", "HEAD", "OPTIONS"} and (
            headers.get(b"sec-fetch-site") == b"cross-site" or (origin and not same_origin)
        ):
            await JSONResponse({"detail": "Cross-origin mutations are not allowed"}, 403)(
                scope, receive, send
            )
            return
        try:
            length = int(headers.get(b"content-length", b"0"))
        except ValueError:
            length = MAX_REQUEST_BYTES + 1
        if length > MAX_REQUEST_BYTES:
            await JSONResponse({"detail": "Request exceeds 16 MiB"}, 413)(scope, receive, send)
            return
        body = bytearray()
        while True:
            message = await receive()
            if message["type"] == "http.disconnect":
                return
            body.extend(message.get("body", b""))
            if len(body) > MAX_REQUEST_BYTES:
                await JSONResponse({"detail": "Request exceeds 16 MiB"}, 413)(scope, receive, send)
                return
            if not message.get("more_body", False):
                break
        consumed = False

        async def replay() -> Message:
            nonlocal consumed
            if not consumed:
                consumed = True
                return {"type": "http.request", "body": bytes(body), "more_body": False}
            return await receive()

        await self.app(scope, replay, send)
