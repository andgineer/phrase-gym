"""Antigravity (agy) launch and Flash model fallback copied from news-recap's runner."""

import json
import logging
import re
import shlex
import subprocess
import threading
import time
from collections.abc import Callable
from pathlib import Path

from phrase_gym.coach.agents.antigravity_errors import AGY_MODEL_REJECTED, read_stderr_safe
from phrase_gym.coach.agents.subprocess import SubprocessResult

logger = logging.getLogger(__name__)


_USAGE_BREAKDOWN_KEYS = (
    "input_tokens",
    "output_tokens",
    "thinking_tokens",
    "cache_read_tokens",
    "total_tokens",
)


def _unwrap_agy_envelope(result: SubprocessResult) -> dict[str, int] | None:
    """Replace agy's ``--output-format json`` envelope in stdout with its response text.

    The envelope's error goes to stderr, where quota/auth detection looks.
    """
    try:
        envelope = json.loads(result.stdout_path.read_text("utf-8"))
    except (OSError, ValueError):
        return None
    if not isinstance(envelope, dict) or "response" not in envelope:
        return None
    result.stdout_path.write_text(envelope.get("response") or "", "utf-8")
    if error := envelope.get("error"):
        with result.stderr_path.open("a", encoding="utf-8") as f:
            f.write(f"\n{error}\n")
    usage = envelope.get("usage")
    if not isinstance(usage, dict):
        return None
    return {k: v for k, v in usage.items() if k in _USAGE_BREAKDOWN_KEYS and isinstance(v, int)}


_EFFORT_LEVELS = ("minimal", "low", "medium", "high", "xhigh", "max")
_MODELS_LIST_TIMEOUT = 60

# Rejected model flags → replacement, kept for the rest of the process so later
# launches skip the failing attempt.
_agy_model_switches: dict[str, str] = {}
_agy_switch_lock = threading.Lock()


def launch_agy(
    launch: Callable[[str], SubprocessResult],
    model: str,
    command_template: str,
    step_name: str,
) -> tuple[SubprocessResult, dict[str, int] | None]:
    """Run agy once, and once more on a listed Flash model if it rejects *model*."""
    model = _agy_model_switches.get(model, model)
    result = launch(model)
    usage = _unwrap_agy_envelope(result)
    if _agy_rejected_model(result):
        replacement = _switch_agy_model(model, command_template, step_name)
        if replacement:
            result = launch(replacement)
            usage = _unwrap_agy_envelope(result)
    return result, usage


def _agy_rejected_model(result: SubprocessResult) -> bool:
    return result.exit_code != 0 and bool(
        AGY_MODEL_REJECTED.search(read_stderr_safe(result.stderr_path)),
    )


def _switch_agy_model(rejected: str, command_template: str, step_name: str) -> str | None:
    with _agy_switch_lock:
        if rejected in _agy_model_switches:
            return _agy_model_switches[rejected]
        model_id = _cheapest_flash_model(shlex.split(command_template)[0])
        replacement = f"--model {model_id}" if model_id else None
        if replacement is None or replacement == rejected:
            logger.error(
                "%s: antigravity rejected %r and lists no other Flash model",
                step_name,
                rejected,
            )
            return None
        _agy_model_switches[rejected] = replacement
        logger.warning(
            "%s: antigravity rejected %r — switching to %r for the rest of this run",
            step_name,
            rejected,
            replacement,
        )
        return replacement


def _cheapest_flash_model(agy: str) -> str | None:
    """Pick from ``agy models`` the lowest-effort Flash model, newest first, Lite last."""
    try:
        listing = subprocess.run(  # noqa: S603
            [agy, "models"],
            capture_output=True,
            text=True,
            timeout=_MODELS_LIST_TIMEOUT,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    model_ids = [
        line.split("\t", 1)[0].strip() for line in listing.stdout.splitlines() if "\t" in line
    ]
    flash = [m for m in model_ids if "flash" in m]
    return min(flash, key=_flash_cost_rank) if flash else None


def _flash_cost_rank(model_id: str) -> tuple[int, bool, tuple[int, ...], str]:
    suffix = model_id.rsplit("-", 1)[-1]
    # A name without an effort suffix runs at agy's default thinking level.
    effort = _EFFORT_LEVELS.index(suffix if suffix in _EFFORT_LEVELS else "medium")
    version = re.search(r"\d+(?:\.\d+)*", model_id)
    newest_first = tuple(-int(n) for n in version.group().split(".")) if version else ()
    return effort, "lite" in model_id, newest_first, model_id


def inject_agy_log_file(run_args: str | list[str], log_path: Path) -> str | list[str]:
    """Redirect agy's internal log to our stderr file so quota/auth errors are captured."""
    flag = "--log-file"
    if isinstance(run_args, list):
        if flag not in run_args:
            run_args = [run_args[0], flag, log_path.as_posix(), *run_args[1:]]
        return run_args
    if flag not in run_args:
        head, _, tail = run_args.partition(" ")
        run_args = f"{head} {flag} {shlex.quote(log_path.as_posix())} {tail}"
    return run_args


# Coach's own pacing from the routing decision: one launch at a time, starts spaced apart.
_launch_lock = threading.Lock()
_last_launch = [float("-inf")]


def paced[T](launch: Callable[[], T], delay: float, stop_event: threading.Event) -> T:
    with _launch_lock:
        wait = _last_launch[0] + delay - time.monotonic()
        if wait > 0:
            stop_event.wait(wait)
        _last_launch[0] = time.monotonic()
        return launch()
