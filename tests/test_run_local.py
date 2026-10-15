import json
import shlex
import socket
from types import SimpleNamespace
from urllib.error import HTTPError
from urllib.request import Request, urlopen

import pytest
from invoke import Exit

from scripts import run_local
from tasks import namespace


HOSTNAME = "trainer.example.ts.net"


@pytest.fixture
def runtime(tmp_path, monkeypatch):
    client = tmp_path / "client"
    client.mkdir()
    (client / "index.html").write_text("<h1>Phrase Gym</h1>")
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path / "data"))
    monkeypatch.setenv("PHRASE_GYM_HOSTS", "127.0.0.1,localhost")
    monkeypatch.delenv("PHRASE_GYM_PREPARATION_CONFIG", raising=False)
    monkeypatch.setattr(run_local, "tailscale_command", lambda: "/test app/tailscale")
    return client


@pytest.mark.parametrize("interrupted", [False, True])
def test_run_serves_reserved_port_with_tailnet_host_and_stops(runtime, interrupted):
    commands = []
    port = None

    class Runner:
        def run(self, command, **_options):
            nonlocal port
            args = shlex.split(command)
            commands.append(args)
            if args[1] == "status":
                return SimpleNamespace(
                    stdout=json.dumps(
                        {"BackendState": "Running", "Self": {"DNSName": HOSTNAME + "."}}
                    )
                )
            assert args[:3] == ["/test app/tailscale", "serve", "--https=443"]
            address = args[3]
            port = int(address.rsplit(":", 1)[1])
            assert port != 8000
            with socket.socket() as competing:
                with pytest.raises(OSError):
                    competing.bind(("127.0.0.1", port))
            request = Request(address + "/api/v1/settings", headers={"Host": HOSTNAME})
            with urlopen(request, timeout=5) as response:
                assert response.status == 200
            update = Request(
                address + "/api/v1/settings",
                data=b'{"native_language":"ru"}',
                headers={
                    "Host": HOSTNAME,
                    "Origin": "https://" + HOSTNAME,
                    "Content-Type": "application/json",
                },
                method="PUT",
            )
            with urlopen(update, timeout=5) as response:
                assert json.load(response)["native_language"] == "ru"
            with pytest.raises(HTTPError) as rejected:
                urlopen(
                    Request(address + "/api/v1/settings", headers={"Host": "untrusted.example"}),
                    timeout=5,
                )
            assert rejected.value.code == 400
            if interrupted:
                raise KeyboardInterrupt
            return SimpleNamespace(stdout="")

    if interrupted:
        with pytest.raises(KeyboardInterrupt):
            run_local.run_local(Runner(), client_dir=str(runtime))
    else:
        run_local.run_local(Runner(), client_dir=str(runtime))
    assert commands[0] == ["/test app/tailscale", "status", "--json"]
    assert len(commands) == 2
    assert port is not None
    with socket.socket() as connection:
        assert connection.connect_ex(("127.0.0.1", port)) != 0


def test_run_requires_built_client_before_tailscale(tmp_path):
    with pytest.raises(Exit, match="Build the client first"):
        run_local.run_local(SimpleNamespace(), client_dir=str(tmp_path))


def test_run_build_rebuilds_default_client_before_tailscale(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(run_local, "tailscale_command", lambda: "tailscale")
    commands = []

    def command(args, **_options):
        commands.append(args)
        if args == "npm --prefix frontend run build":
            client = tmp_path / run_local.DEFAULT_CLIENT_DIR
            client.mkdir(parents=True)
            (client / "index.html").write_text("<h1>Phrase Gym</h1>")
        return SimpleNamespace(stdout="{}")

    with pytest.raises(Exit, match="Connect Tailscale"):
        run_local.run_local(SimpleNamespace(run=command), build=True)
    assert commands == ["npm --prefix frontend run build", "tailscale status --json"]


def test_run_build_rejects_other_client_dir(runtime):
    with pytest.raises(Exit, match="--build only rebuilds"):
        run_local.run_local(SimpleNamespace(), client_dir=str(runtime), build=True)


def test_run_requires_connected_tailscale(runtime):
    runner = SimpleNamespace(run=lambda *_args, **_kwargs: SimpleNamespace(stdout="{}"))
    with pytest.raises(Exit, match="Connect Tailscale"):
        run_local.run_local(runner, client_dir=str(runtime))


def test_run_does_not_share_if_server_startup_fails(runtime, monkeypatch):
    commands = []

    def command(args, **_options):
        commands.append(shlex.split(args))
        return SimpleNamespace(
            stdout=json.dumps({"BackendState": "Running", "Self": {"DNSName": HOSTNAME}})
        )

    monkeypatch.setattr(
        run_local.uvicorn,
        "Server",
        lambda _config: SimpleNamespace(started=False, run=lambda **_kwargs: None),
    )
    with pytest.raises(Exit, match="failed to start"):
        run_local.run_local(SimpleNamespace(run=command), client_dir=str(runtime))
    assert len(commands) == 1


def test_invoke_registers_run_with_data_client_and_build_options():
    assert namespace["run"].body.__name__ == "run"
    assert set(namespace["run"].help) == {"build", "client_dir", "data_dir"}


def test_tailscale_cli_prefers_path_installation(monkeypatch):
    monkeypatch.setattr(run_local.shutil, "which", lambda _name: "/installed/tailscale")
    assert run_local.tailscale_command() == "/installed/tailscale"


@pytest.mark.parametrize("installed", [False, True])
def test_tailscale_cli_uses_mac_app_or_reports_missing_installation(monkeypatch, installed):
    monkeypatch.setattr(run_local.shutil, "which", lambda _name: None)
    monkeypatch.setattr(run_local.Path, "is_file", lambda _path: installed)
    if installed:
        assert (
            run_local.tailscale_command() == "/Applications/Tailscale.app/Contents/MacOS/Tailscale"
        )
    else:
        with pytest.raises(Exit, match="Install Tailscale"):
            run_local.tailscale_command()
