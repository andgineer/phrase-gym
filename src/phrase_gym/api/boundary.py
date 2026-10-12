from urllib.parse import urlsplit

from fastapi.responses import JSONResponse

MAX_REQUEST_BYTES = 24 * 1024 * 1024


class LocalBoundary:
    """Reject cross-site mutations and bound memory before JSON parsing."""

    def __init__(self, app, allowed_origins: tuple[str, ...]):
        self.app = app
        self.allowed_origins = allowed_origins

    def cross_origin(self, scope, headers) -> bool:
        if scope["method"] in ("GET", "HEAD", "OPTIONS"):
            return False
        origin = headers.get(b"origin", b"").decode()
        origin_url = urlsplit(origin)
        same_origin = (
            origin_url.netloc == headers.get(b"host", b"").decode()
            and origin_url.scheme == scope["scheme"]
        )
        cross_site = headers.get(b"sec-fetch-site") == b"cross-site"
        return cross_site or bool(
            origin and not same_origin and origin not in self.allowed_origins,
        )

    @staticmethod
    def too_large(headers) -> bool:
        try:
            return int(headers.get(b"content-length", b"0")) > MAX_REQUEST_BYTES
        except ValueError:
            return True

    async def __call__(self, scope, receive, send):
        if scope["type"] != "http":
            await self.app(scope, receive, send)
            return
        headers = dict(scope["headers"])
        if self.cross_origin(scope, headers):
            await JSONResponse({"detail": "Cross-origin mutations are not allowed"}, 403)(
                scope,
                receive,
                send,
            )
            return
        if self.too_large(headers):
            await JSONResponse({"detail": "Request exceeds 24 MiB"}, 413)(scope, receive, send)
            return
        body = bytearray()
        while True:
            message = await receive()
            if message["type"] == "http.disconnect":
                return
            body.extend(message.get("body", b""))
            if len(body) > MAX_REQUEST_BYTES:
                await JSONResponse({"detail": "Request exceeds 24 MiB"}, 413)(scope, receive, send)
                return
            if not message.get("more_body", False):
                break
        consumed = False

        async def replay():
            nonlocal consumed
            if not consumed:
                consumed = True
                return {"type": "http.request", "body": bytes(body), "more_body": False}
            return await receive()

        await self.app(scope, replay, send)
