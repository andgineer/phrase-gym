import sys

import pytest

from phrase_gym.server import main as cli


def test_run_dispatches_loopback_and_data_dir(monkeypatch, tmp_path):
    called = []
    monkeypatch.setattr(cli.uvicorn, "run", lambda *args, **kwargs: called.append((args, kwargs)))
    monkeypatch.setattr(sys, "argv", ["phrase-gym", "run", "--data-dir", str(tmp_path)])
    cli.main()
    assert called == [
        (
            ("phrase_gym.server.app:create_app",),
            {"factory": True, "host": "127.0.0.1", "port": 8000},
        )
    ]


def test_coach_dispatches_to_separate_api_worker(monkeypatch):
    called = []
    monkeypatch.setattr(cli, "run_coach", lambda url, once: called.append((url, once)))
    monkeypatch.setattr(
        sys, "argv", ["phrase-gym", "coach", "--url", "http://127.0.0.1:8941", "--once"]
    )
    cli.main()
    assert called == [("http://127.0.0.1:8941", True)]


@pytest.mark.parametrize("command", ["install-coach", "uninstall-coach", "deploy"])
def test_future_commands_fail_clearly(monkeypatch, capsys, command):
    monkeypatch.setattr(sys, "argv", ["phrase-gym", command])
    with pytest.raises(SystemExit) as error:
        cli.main()
    assert error.value.code == 2
    assert f"{command} is not available yet" in capsys.readouterr().err


def test_non_loopback_host_is_rejected(monkeypatch, capsys):
    monkeypatch.setattr(sys, "argv", ["phrase-gym", "run", "--host", "0.0.0.0"])
    with pytest.raises(SystemExit):
        cli.main()
    assert "only listens on loopback" in capsys.readouterr().err
