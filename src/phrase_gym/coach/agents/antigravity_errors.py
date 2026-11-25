"""Antigravity (agy) failure classification copied from news-recap's runner."""

import re
from collections.abc import Callable
from datetime import datetime, timedelta
from pathlib import Path

from phrase_gym.coach.agents.subprocess import SubprocessResult


def read_stderr_safe(path: Path) -> str:
    try:
        return path.read_text("utf-8", errors="replace").strip()
    except OSError:
        return ""


# agy logs "You are not logged into Antigravity" on every start, before it
# refreshes the keyring token — the message only means anything if the run never
# went on to report a successful auth.
_AUTH_SUCCEEDED = re.compile(
    r"silent auth succeeded|authenticated successfully|authenticated via keyring",
    re.IGNORECASE,
)

# agy reports the remaining quota window as "Resets in 103h45m22s".
_RESET_IN_RE = re.compile(
    r"Resets? in\s+(?:(\d+)d)?(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?",
    re.IGNORECASE,
)
_DAILY_WINDOW = timedelta(hours=24)

AGY_MODEL_REJECTED = re.compile(r"invalid model selection|requires --effort", re.IGNORECASE)


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
    delta = _parse_reset_in(text)
    if delta is None:
        return (
            "Antigravity quota exhausted, no reset time reported — "
            "switch Coach (--cli claude|codex) or upgrade the plan"
        )
    reset_at = datetime.now().astimezone() + delta
    window = "weekly" if delta > _DAILY_WINDOW else "daily"
    advice = (
        "retrying today will not help"
        if delta > _DAILY_WINDOW
        else "retry after the reset or reduce the workload"
    )
    return (
        f"Antigravity {window} quota exhausted — resets in {_format_duration(delta)}, "
        f"around {reset_at:%Y-%m-%d %H:%M %Z}; {advice}. "
        "Switch Coach (--cli claude|codex) or upgrade the Antigravity plan"
    )


_KNOWN_ERRORS: list[tuple[re.Pattern[str], str | Callable[[str], str], re.Pattern[str] | None]] = [
    (
        re.compile(r"Individual quota reached|upgrade your subscription", re.IGNORECASE),
        _quota_summary,
        None,
    ),
    (
        re.compile(r"not logged into Antigravity|You are not logged in", re.IGNORECASE),
        "Not logged into Antigravity — run: agy login",
        _AUTH_SUCCEEDED,
    ),
    (
        AGY_MODEL_REJECTED,
        "Invalid model/effort selection — check the agent's --model flags",
        None,
    ),
    (
        re.compile(r"RetryableQuotaError:.*exhausted your capacity", re.IGNORECASE),
        "Antigravity API quota exhausted (rate limit) — reduce parallelism or wait",
        None,
    ),
    (
        # A bare 429 also matches PIDs and trace ids in agent logs.
        re.compile(
            r"rate.?limit|too many requests|\b(status|code|http)\b\W{0,10}429",
            re.IGNORECASE,
        ),
        "API rate limit hit — reduce parallelism or wait",
        None,
    ),
]


def _summarise_stderr(text: str) -> str | None:
    for pattern, summary, negated_by in _KNOWN_ERRORS:
        if pattern.search(text) and not (negated_by and negated_by.search(text)):
            return summary(text) if callable(summary) else summary
    return None


def _summarise_output(result: SubprocessResult) -> str | None:
    for path in (result.stderr_path, result.stdout_path):
        text = read_stderr_safe(path)
        if text:
            summary = _summarise_stderr(text)
            if summary:
                return summary
    return None


def agy_failure(result: SubprocessResult, agent: str) -> str | None:
    if result.exit_code == 0:
        try:
            stdout_empty = not result.stdout_path.read_text("utf-8").strip()
        except OSError:
            stdout_empty = True
        if not stdout_empty:
            return None
        summary = _summarise_output(result)
        detail = f": {summary}" if summary else " — run the agent interactively to check quota/auth"
        return f"{agent}: no output produced{detail}"
    summary = _summarise_output(result)
    if result.timed_out:
        return f"{agent}: agent timed out{f' ({summary})' if summary else ''}"
    if result.output_exceeded:
        return f"{agent}: agent exceeded the output limit"
    if summary:
        return f"{agent}: {summary}"
    return f"{agent}: agent exit code {result.exit_code}"
