import re
from datetime import timedelta
from pathlib import Path
from unittest.mock import patch

import pytest

from phrase_gym.coach.agents.ai_agent import (
    _format_duration,
    _inject_skip_git_flag,
    _parse_reset_in,
    _run_agent_cli,
    _summarise_stderr,
    agent_failure,
)
from phrase_gym.coach.agents.subprocess import SubprocessResult

ERROR_LINE = re.compile(r"^ERROR:.*$", re.M)


def _fake_result(tmp_path: Path, **changes: object) -> SubprocessResult:
    fields = {
        "exit_code": 0,
        "timed_out": False,
        "stdout_path": tmp_path / "step-stdout.log",
        "stderr_path": tmp_path / "step-stderr.log",
    }
    return SubprocessResult(**{**fields, **changes})  # type: ignore[arg-type]


def _run_and_capture_env(tmp_path: Path, **kwargs) -> dict[str, str]:
    captured: dict[str, str] = {}

    def fake_run_subprocess(**kw):
        captured.update(kw["env"])
        return _fake_result(tmp_path)

    with patch("phrase_gym.coach.agents.ai_agent.run_subprocess", side_effect=fake_run_subprocess):
        _run_agent_cli(
            prompt_file=tmp_path / "step-prompt.md",
            stdout_path=tmp_path / "step-stdout.log",
            stderr_path=tmp_path / "step-stderr.log",
            cwd=tmp_path,
            timeout_seconds=10,
            command_template="echo {model} -- {prompt_file}",
            values={},
            **kwargs,
        )
    return captured


def test_api_keys_removed_by_default(tmp_path, monkeypatch):
    monkeypatch.setenv("ANTHROPIC_API_KEY", "secret-anthropic")
    env = _run_and_capture_env(tmp_path, api_key_vars=["ANTHROPIC_API_KEY"])
    assert "ANTHROPIC_API_KEY" not in env


def test_unset_key_missing_from_env_is_not_an_error(tmp_path):
    env = _run_and_capture_env(tmp_path, api_key_vars=["ANTHROPIC_API_KEY"])
    assert "ANTHROPIC_API_KEY" not in env


def test_only_allowlisted_variables_reach_the_agent(tmp_path, monkeypatch):
    for name in ("CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT", "ANTHROPIC_BASE_URL", "UNRELATED"):
        monkeypatch.setenv(name, "set")
    env = _run_and_capture_env(tmp_path, api_key_vars=[])
    assert {"HOME", "PATH"} <= set(env)
    assert not {"CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT", "ANTHROPIC_BASE_URL", "UNRELATED"} & set(
        env
    )


def test_command_line_is_kept_beside_the_logs(tmp_path):
    _run_and_capture_env(tmp_path, api_key_vars=[])
    assert (tmp_path / "step-command.txt").read_text() == f"echo -- {tmp_path}/step-prompt.md"


def test_inject_skip_git_list_with_exec() -> None:
    inp = ["codex", "exec", "--model", "o3"]
    out = _inject_skip_git_flag(inp)
    assert out == ["codex", "exec", "--skip-git-repo-check", "--model", "o3"]
    assert inp == ["codex", "exec", "--model", "o3"]


def test_inject_skip_git_list_no_exec() -> None:
    inp = ["codex", "--model", "o3"]
    out = _inject_skip_git_flag(inp)
    assert out == ["codex", "--skip-git-repo-check", "--model", "o3"]


def test_inject_skip_git_list_already_present() -> None:
    inp = ["codex", "exec", "--skip-git-repo-check", "--model", "o3"]
    out = _inject_skip_git_flag(inp)
    assert out == inp


def test_inject_skip_git_string() -> None:
    s = "codex exec --model o3"
    out = _inject_skip_git_flag(s)
    assert out == "codex exec --skip-git-repo-check --model o3"


def test_inject_skip_git_string_already_present() -> None:
    s = "codex exec --skip-git-repo-check --model o3"
    out = _inject_skip_git_flag(s)
    assert out == s


def test_summarise_ignores_429_inside_pids_and_trace_ids() -> None:
    text = "I0731 16:08:52.167611  4291 quota_manager.go:44] Trace: 0x429fe3d6590b3df6\n"
    assert _summarise_stderr(text) is None


def test_summarise_detects_real_429_status() -> None:
    text = "request failed with status 429\n"
    assert _summarise_stderr(text) == "rate limit hit; submit the text again later"


def test_parse_reset_in_agy_format() -> None:
    assert _parse_reset_in("Resets in 103h45m22s.") == timedelta(hours=103, minutes=45, seconds=22)
    assert _parse_reset_in("Resets in 2d3h.") == timedelta(days=2, hours=3)
    assert _parse_reset_in("no reset here") is None


def test_format_duration() -> None:
    assert _format_duration(timedelta(hours=103, minutes=45)) == "4d 7h 45m"
    assert _format_duration(timedelta(minutes=45)) == "45m"
    assert _format_duration(timedelta(seconds=20)) == "less than a minute"


@pytest.mark.parametrize(
    ("text", "when"),
    [
        ("You've hit your 5-hour limit · resets 6pm (Europe/Belgrade)", "6pm (Europe/Belgrade)"),
        (
            "ERROR: You've hit your usage limit. Try again at Nov 13th, 2026 4:58 PM.",
            "Nov 13th, 2026 4:58 PM",
        ),
    ],
)
def test_summarise_usage_limit_names_the_stated_reset(text: str, when: str) -> None:
    assert _summarise_stderr(text) == f"usage limit reached; submit the text again after {when}"


def test_summarise_usage_limit_with_a_relative_reset() -> None:
    summary = _summarise_stderr("Individual quota reached. Resets in 3h20m.")
    assert summary is not None
    assert "(in 3h 20m)" in summary


def test_agent_failure_reads_only_the_cli_error_lines(tmp_path: Path) -> None:
    result = _fake_result(tmp_path, exit_code=1)
    result.stdout_path.write_text("")
    result.stderr_path.write_text("a quotation; not logged in? unauthorized\nnothing else\n")
    assert agent_failure(result, "Codex", ERROR_LINE) == "Codex: agent exit code 1"
    result.stderr_path.write_text("echo of prompt\nERROR: Not logged in\n")
    assert agent_failure(result, "Codex", ERROR_LINE) == (
        "Codex: not logged in; log in to the CLI in a terminal, then submit the text again"
    )


def test_agent_failure_reports_timeout_and_empty_output(tmp_path: Path) -> None:
    timed_out = _fake_result(tmp_path, exit_code=124, timed_out=True)
    timed_out.stdout_path.write_text("")
    timed_out.stderr_path.write_text("")
    assert agent_failure(timed_out, "Claude", ERROR_LINE) == "Claude: agent timed out"
    empty = _fake_result(tmp_path)
    assert agent_failure(empty, "Claude", ERROR_LINE) == "Claude: no output produced"
    empty.stdout_path.write_text("{}")
    assert agent_failure(empty, "Claude", ERROR_LINE) is None


def test_custom_cli_homes_reach_the_agent_but_other_claude_state_does_not(tmp_path, monkeypatch):
    monkeypatch.setenv("CODEX_HOME", "/custom/codex")
    monkeypatch.setenv("CLAUDE_CONFIG_DIR", "/custom/claude")
    monkeypatch.setenv("CLAUDE_CODE_SSE_PORT", "1234")
    monkeypatch.setenv("ANTHROPIC_MODEL", "pinned")
    env = _run_and_capture_env(tmp_path, api_key_vars=[])
    assert env["CODEX_HOME"] == "/custom/codex"
    assert env["CLAUDE_CONFIG_DIR"] == "/custom/claude"
    assert not {"CLAUDE_CODE_SSE_PORT", "ANTHROPIC_MODEL"} & set(env)


def test_codex_given_as_a_full_path_still_skips_the_git_check(tmp_path):
    captured: list[list[str]] = []

    def fake_run_subprocess(**kw):
        captured.append(kw["run_args"])
        return _fake_result(tmp_path)

    with patch("phrase_gym.coach.agents.ai_agent.run_subprocess", side_effect=fake_run_subprocess):
        _run_agent_cli(
            prompt_file=tmp_path / "step-prompt.md",
            stdout_path=tmp_path / "step-stdout.log",
            stderr_path=tmp_path / "step-stderr.log",
            cwd=tmp_path,
            timeout_seconds=10,
            command_template='/opt/homebrew/bin/codex exec {model} "{prompt_file}"',
            values={},
        )
    assert captured[0][:3] == ["/opt/homebrew/bin/codex", "exec", "--skip-git-repo-check"]


def test_unrecognised_failures_keep_the_cli_error_text_but_not_the_prompt(tmp_path: Path) -> None:
    claude_errors = re.compile(r'^(?:Error|API Error)\b.*$|^.*"is_error":\s*true.*$', re.M)
    result = _fake_result(tmp_path, exit_code=1)
    result.stdout_path.write_text("")
    result.stderr_path.write_text(
        "prompt echo: please quote\nAPI Error: 500 Internal server error\n"
    )
    assert agent_failure(result, "Claude", claude_errors) == (
        "Claude: agent exit code 1: API Error: 500 Internal server error"
    )
    result.stderr_path.write_text("")
    result.stdout_path.write_text(
        '{"type": "result", "is_error": true, "result": "Something unexpected broke"}\n'
    )
    assert agent_failure(result, "Claude", claude_errors) == (
        "Claude: agent exit code 1: Something unexpected broke"
    )
