"""Run a loopback development server with a foreground Tailscale proxy."""

import json
import shlex
import shutil
import socket
import threading
import time
from dataclasses import replace
from pathlib import Path

import uvicorn
from invoke import Context, Exit

from phrase_gym.api.app import create_app
from phrase_gym.config import Config

EXCLUDED_PORT = 8000
DEFAULT_CLIENT_DIR = "frontend/dist"
MAC_TAILSCALE = "/Applications/Tailscale.app/Contents/MacOS/Tailscale"


def tailscale_command() -> str:
    installed = shutil.which("tailscale")
    if installed:
        return installed
    if Path(MAC_TAILSCALE).is_file():
        return MAC_TAILSCALE
    raise Exit("Install Tailscale and sign in before running inv run.", code=1)


def reserve_port() -> socket.socket:
    while True:
        listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        listener.bind(("127.0.0.1", 0))
        if listener.getsockname()[1] != EXCLUDED_PORT:
            return listener
        listener.close()


def run_local(
    c: Context,
    data_dir: str = "",
    client_dir: str = DEFAULT_CLIENT_DIR,
    build: bool = False,
) -> None:
    client = Path(client_dir)
    if build:
        if client != Path(DEFAULT_CLIENT_DIR):
            raise Exit(f"--build only rebuilds {DEFAULT_CLIENT_DIR}; omit --client-dir.", code=1)
        c.run("npm --prefix frontend run build")
    if not (client / "index.html").is_file():
        raise Exit("Build the client first: npm --prefix frontend run build", code=1)
    tailscale = tailscale_command()
    result = c.run(shlex.join([tailscale, "status", "--json"]), hide=True)
    if result is None:
        raise Exit("Tailscale status did not complete.", code=1)
    status = json.loads(result.stdout)
    hostname = status.get("Self", {}).get("DNSName", "").rstrip(".")
    if status.get("BackendState") != "Running" or not hostname:
        raise Exit("Connect Tailscale and enable MagicDNS before running inv run.", code=1)
    runtime = Config.from_env()
    runtime = replace(
        runtime,
        data_dir=Path(data_dir) if data_dir else runtime.data_dir,
        client_dir=client,
        allowed_hosts=tuple(dict.fromkeys([*runtime.allowed_hosts, hostname])),
        allowed_origins=tuple(dict.fromkeys([*runtime.allowed_origins, f"https://{hostname}"])),
    )
    with reserve_port() as listener:
        port = listener.getsockname()[1]
        server = uvicorn.Server(
            uvicorn.Config(
                create_app(runtime),
                proxy_headers=False,
                timeout_graceful_shutdown=5,
            ),
        )
        # Hand the bound socket to Uvicorn so another process cannot steal the port.
        worker = threading.Thread(
            target=server.run,
            kwargs={"sockets": [listener]},
            name="phrase-gym-server",
            daemon=True,
        )
        worker.start()
        try:
            deadline = time.monotonic() + 15
            while not server.started:
                if not worker.is_alive() or time.monotonic() >= deadline:
                    raise Exit("Phrase Gym failed to start; see the server log.", code=1)
                time.sleep(0.05)
            print(f"Local app: http://127.0.0.1:{port}/learn/en", flush=True)
            print(f"Tailscale app: https://{hostname}/learn/en", flush=True)
            c.run(shlex.join([tailscale, "serve", "--https=443", f"http://127.0.0.1:{port}"]))
        finally:
            server.should_exit = True
            worker.join(timeout=10)
