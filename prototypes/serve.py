"""Serve the disposable P1 prototypes; never expose the source tree."""

import argparse
import hmac
import http.cookies
import ipaddress
import secrets
import ssl
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

ROOT = Path(__file__).resolve().parent
PUBLIC_DIRS = {"drill-flow", "audio-probe", "shared", "media"}
PUBLIC_FILES = {"sw.js", "manifest.webmanifest"}


class PrototypeHandler(SimpleHTTPRequestHandler):
    """Read-only local prototype assets with an optional private TLS boundary."""

    token: str | None = None
    tls = False

    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT), **kwargs)

    def do_GET(self) -> None:  # noqa: N802 - stdlib HTTP handler interface.
        """Authenticate private access and resolve language bookmark routes."""
        path = urlsplit(self.path)
        if self.token:
            supplied = parse_qs(path.query).get("access", [""])[0]
            if supplied and hmac.compare_digest(supplied, self.token):
                self.send_response(303)
                self.send_header(
                    "Set-Cookie",
                    f"pg_probe={self.token}; HttpOnly; Secure; SameSite=Strict; Path=/",
                )
                self.send_header("Location", path.path)
                self.send_header("Cache-Control", "no-store")
                self.end_headers()
                return
            cookies = http.cookies.SimpleCookie()
            try:
                cookies.load(self.headers.get("Cookie", ""))
            except http.cookies.CookieError:
                self.send_error(403)
                return
            cookie = cookies.get("pg_probe")
            if not cookie or not hmac.compare_digest(cookie.value, self.token):
                self.send_error(403, "Open the private access link printed by the server.")
                return
        if path.path == "/" or path.path.startswith("/learn/"):
            self.path = "/drill-flow/index.html"
        elif path.path.rstrip("/") == "/audio-probe":
            self.path = "/audio-probe/index.html"
        resolved = Path(self.translate_path(self.path)).resolve()
        if not resolved.is_relative_to(ROOT):
            self.send_error(404)
            return
        relative = resolved.relative_to(ROOT)
        if not relative.parts or (
            relative.parts[0] not in PUBLIC_DIRS and str(relative) not in PUBLIC_FILES
        ):
            self.send_error(404)
            return
        if resolved.is_dir():
            self.send_error(404)
            return
        super().do_GET()

    def do_HEAD(self) -> None:  # noqa: N802 - stdlib HTTP handler interface.
        """Avoid bypassing private checks through the default HEAD handler."""
        self.send_error(405)

    def end_headers(self) -> None:
        """Prevent sniffing and third-party connections; allow locally stored blobs."""
        self.send_header("Referrer-Policy", "no-referrer")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Permissions-Policy", "camera=(), microphone=(self)")
        self.send_header(
            "Content-Security-Policy",
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; "
            "media-src 'self' blob:; img-src 'self' blob:; connect-src 'self'; "
            "object-src 'none'; base-uri 'none'; frame-ancestors 'none'",
        )
        super().end_headers()

    def log_message(self, format: str, *args) -> None:
        """Do not log access tokens or private request URLs."""
        del format, args


def main() -> None:
    """Keep HTTP on loopback; require TLS and a generated access key elsewhere."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8871)
    parser.add_argument("--cert", type=Path)
    parser.add_argument("--key", type=Path)
    args = parser.parse_args()
    try:
        loopback = ipaddress.ip_address(args.host).is_loopback
    except ValueError:
        loopback = args.host == "localhost"
    if bool(args.cert) != bool(args.key):
        parser.error("Supply both --cert and --key.")
    if not loopback and not args.cert:
        parser.error("Phone/LAN access requires a trusted HTTPS certificate and key.")
    PrototypeHandler.token = None if loopback else secrets.token_urlsafe(32)
    server = ThreadingHTTPServer((args.host, args.port), partial(PrototypeHandler))
    if args.cert:
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.minimum_version = ssl.TLSVersion.TLSv1_2
        context.load_cert_chain(args.cert, args.key)
        server.socket = context.wrap_socket(server.socket, server_side=True)
    scheme = "https" if args.cert else "http"
    suffix = f"?access={PrototypeHandler.token}" if PrototypeHandler.token else ""
    print(f"Phrase Gym P1: {scheme}://{args.host}:{args.port}/learn/en{suffix}", flush=True)
    print("Audio probe: /audio-probe/ · Ctrl-C to stop", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
