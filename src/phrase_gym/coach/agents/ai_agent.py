import json
import os
import re
import shlex
import threading
from collections.abc import Callable
from datetime import datetime, timedelta
from pathlib import Path

from phrase_gym.coach.agents.command_template import build_run_args
from phrase_gym.coach.agents.config import INHERITED_ENVIRONMENT
from phrase_gym.coach.agents.subprocess import SubprocessError, SubprocessResult, run_subprocess

MAX_OUTPUT_BYTES = 8 * 1024 * 1024


def _read_stderr_safe(path: Path) -> str:
    try:
        return path.read_text("utf-8", errors="replace").strip()
    except OSError:
        return ""


# agy reports the remaining quota window as "Resets in 103h45m22s".
_RESET_IN_RE = re.compile(
    r"Resets? in\s+(?:(\d+)d)?(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?",
    re.IGNORECASE,
)
# Claude states "resets 6pm (Europe/Belgrade)"; Codex states "try again at Nov 13th, 4:58 PM".
_RESET_AT_RE = re.compile(r"\bresets (?!in\b)([^·|\"\n]{1,60})|try again at ([^.\"\n]{1,60})", re.I)


def _parse_reset_in(text: str) -> timedelta | None:
    match = _RESET_IN_RE.search(text)
    if match is None or not any(match.groups()):
        return None
    days, hours, minutes, seconds = (int(g or 0) for g in match.groups())
    return timedelta(days=days, hours=hours, minutes=minutes, seconds=seconds)


def _format_duration(delta: timedelta) -> str:
    total = int(delta.total_seconds())
    days, rest = divmod(total, 86400)
    hours, rest = divmod(rest, 3600)
    minutes = rest // 60
    parts = [f"{days}d", f"{hours}h", f"{minutes}m"] if days else [f"{hours}h", f"{minutes}m"]
    return " ".join(p for p in parts if not p.startswith("0")) or "less than a minute"


def _quota_summary(text: str) -> str:
    if match := _RESET_AT_RE.search(text):
        when = (match[1] or match[2]).strip()
    elif delta := _parse_reset_in(text):
        reset_at = datetime.now().astimezone() + delta
        when = f"{reset_at:%Y-%m-%d %H:%M %Z} (in {_format_duration(delta)})"
    else:
        return "usage limit reached; submit the text again later"
    return f"usage limit reached; submit the text again after {when}"


_KNOWN_ERRORS: list[tuple[re.Pattern[str], str | Callable[[str], str], re.Pattern[str] | None]] = [
    (
        re.compile(
            r"\busage limit\b|\bhit your (?:[\w-]+ )?limit\b|\bquota\b|upgrade your subscription",
            re.IGNORECASE,
        ),
        _quota_summary,
        None,
    ),
    (
        re.compile(
            r"\bnot logged in\b|please run /login|\bcodex login\b|\binvalid api key\b"
            r"|\bunauthorized\b|\boauth token\b.{0,40}\b(?:expired|revoked)\b",
            re.IGNORECASE,
        ),
        "not logged in; log in to the CLI in a terminal, then submit the text again",
        None,
    ),
    (
        re.compile(
            r"credit balance.{0,20}(too low|insufficient)|insufficient.{0,20}(funds|credits)",
            re.IGNORECASE,
        ),
        "credit balance too low; Coach uses only the subscription login",
        None,
    ),
    (
        re.compile(r"OverloadedError|overloaded_error", re.IGNORECASE),
        "service overloaded; submit the text again later",
        None,
    ),
    (
        # A bare 429 also matches PIDs and trace ids in agent logs.
        re.compile(
            r"rate.?limit|too many requests|\b(status|code|http)\b\W{0,10}429",
            re.IGNORECASE,
        ),
        "rate limit hit; submit the text again later",
        None,
    ),
]


def _summarise_stderr(text: str) -> str | None:
    for pattern, summary, negated_by in _KNOWN_ERRORS:
        if pattern.search(text) and not (negated_by and negated_by.search(text)):
            return summary(text) if callable(summary) else summary
    return None


# Transcripts echo the prompt and learner text, so only the CLI's own error lines are read.
def _summarise_output(result: SubprocessResult, error_line: re.Pattern[str]) -> str | None:
    for path in (result.stderr_path, result.stdout_path):
        text = "\n".join(error_line.findall(_read_stderr_safe(path)))
        if text:
            summary = _summarise_stderr(text)
            if summary:
                return summary
    return None


def agent_failure(
    result: SubprocessResult,
    agent: str,
    error_line: re.Pattern[str],
) -> str | None:
    summary = _summarise_output(result, error_line)
    if result.exit_code == 0:
        try:
            stdout_empty = not result.stdout_path.read_text("utf-8").strip()
        except OSError:
            stdout_empty = True
        return (
            f"{agent}: no output produced{f': {summary}' if summary else ''}"
            if stdout_empty
            else None
        )
    if result.timed_out:
        return f"{agent}: agent timed out{f' ({summary})' if summary else ''}"
    if result.output_exceeded:
        return f"{agent}: agent exceeded the output limit"
    if summary:
        return f"{agent}: {summary}"
    detail = _error_detail(result, error_line)
    return f"{agent}: agent exit code {result.exit_code}{f': {detail}' if detail else ''}"


# Only the CLI's own error lines are shown; a JSON result envelope contributes its message.
def _error_detail(result: SubprocessResult, error_line: re.Pattern[str]) -> str:
    for path in (result.stderr_path, result.stdout_path):
        for line in error_line.findall(_read_stderr_safe(path)):
            try:
                envelope = json.loads(line)
            except ValueError:
                envelope = None
            text = line
            if isinstance(envelope, dict):
                text = str(envelope.get("result") or envelope.get("error") or "")
            if text.strip():
                return text.strip()[:300]
    return ""


def _child_env() -> dict[str, str]:
    inherited = {name: os.environ[name] for name in INHERITED_ENVIRONMENT if name in os.environ}
    return {**inherited, "TERM": "dumb", "NO_COLOR": "1"}


def _run_agent_cli(  # noqa: PLR0913
    *,
    prompt_file: Path,
    stdout_path: Path,
    stderr_path: Path,
    cwd: Path,
    timeout_seconds: int,
    command_template: str,
    values: dict[str, str | Path],
    api_key_vars: list[str] | None = None,
    log_label: str = "",
    error_line: re.Pattern[str] | None = None,
    stop_event: threading.Event | None = None,
) -> SubprocessResult:
    run_args, command_head = build_run_args(
        command_template,
        os_name=None,
        model="",
        prompt_file=prompt_file,
        **values,
    )

    # Coach may run Codex from a full path, so its base name decides.
    if Path(command_head).name == "codex":
        run_args = _inject_skip_git_flag(run_args)

    cmd_line = run_args if isinstance(run_args, str) else shlex.join(run_args)
    stdout_path.with_name(stdout_path.name.replace("stdout.log", "command.txt")).write_text(
        cmd_line,
        "utf-8",
    )

    env = _child_env()
    for _key in api_key_vars or []:
        env.pop(_key, None)

    try:
        return run_subprocess(
            run_args=run_args,
            env=env,
            cwd=cwd,
            timeout_seconds=timeout_seconds,
            stdout_path=stdout_path,
            stderr_path=stderr_path,
            log_label=log_label,
            stop_event=stop_event,
            max_output_bytes=MAX_OUTPUT_BYTES,
            error_line=error_line,
        )
    except FileNotFoundError as error:
        raise SubprocessError(
            f"Agent command not found: {command_head}",
            transient=False,
        ) from error
    except OSError as error:
        raise SubprocessError(
            f"Agent failed to start: {error}",
            transient=True,
        ) from error


def _inject_skip_git_flag(run_args: str | list[str]) -> str | list[str]:
    flag = "--skip-git-repo-check"
    if isinstance(run_args, list):
        if flag not in run_args:
            try:
                idx = run_args.index("exec") + 1
            except ValueError:
                idx = 1
            run_args = [*run_args[:idx], flag, *run_args[idx:]]
        return run_args
    if flag not in run_args:
        run_args = run_args.replace("codex exec", f"codex exec {flag}", 1)
    return run_args
