import json

import pytest
from invoke import Context, Result

from scripts.private_access import existing_tailscale_origin


def serve_config():
    return {
        "TCP": {"443": {"HTTPS": True}},
        "Web": {
            "macbook.example.ts.net:443": {"Handlers": {"/": {"Proxy": "http://127.0.0.1:8000"}}}
        },
    }


def detect(monkeypatch, config=None, exit_code=0):
    monkeypatch.setattr("scripts.private_access.shutil.which", lambda _name: "/bin/tailscale")
    context = Context()

    def execute(command, **kwargs):
        assert command == "tailscale serve status --json"
        assert kwargs == {"hide": True, "warn": True, "pty": False}
        return Result(stdout=json.dumps(config), exited=exit_code)

    monkeypatch.setattr(context, "run", execute)
    return existing_tailscale_origin(context)


def test_reuses_existing_exact_private_https_route(monkeypatch):
    assert detect(monkeypatch, serve_config()) == "https://macbook.example.ts.net"


def test_missing_tailscale_does_not_execute_any_command(monkeypatch):
    monkeypatch.setattr("scripts.private_access.shutil.which", lambda _name: None)
    context = Context()
    monkeypatch.setattr(context, "run", lambda *_args, **_kwargs: pytest.fail("unexpected command"))
    assert existing_tailscale_origin(context) == ""


@pytest.mark.parametrize("config", [{}, {"Web": {}}, {"TCP": {"443": {"HTTPS": False}}}])
def test_no_https_route_keeps_local_access(monkeypatch, config):
    assert detect(monkeypatch, config) == ""


def test_unavailable_daemon_keeps_local_access(monkeypatch):
    assert detect(monkeypatch, exit_code=1) == ""


@pytest.mark.parametrize("proxy", ["http://127.0.0.1:9000", "http://other-machine:8000"])
def test_other_servers_are_not_trusted(monkeypatch, proxy):
    config = serve_config()
    config["Web"]["macbook.example.ts.net:443"]["Handlers"]["/"]["Proxy"] = proxy
    assert detect(monkeypatch, config) == ""


def test_public_funnel_is_refused(monkeypatch):
    config = serve_config()
    config["AllowFunnel"] = {"macbook.example.ts.net:443": True}
    with pytest.raises(ValueError, match="disable Funnel"):
        detect(monkeypatch, config)


def test_unrelated_hostname_is_refused(monkeypatch):
    config = serve_config()
    config["Web"]["unrelated.example:443"] = config["Web"].pop("macbook.example.ts.net:443")
    with pytest.raises(ValueError, match="Tailscale origin"):
        detect(monkeypatch, config)
