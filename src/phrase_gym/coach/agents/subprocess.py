import logging
import re
import subprocess
import threading
import time
from dataclasses import dataclass
from pathlib import Path

logger = logging.getLogger(__name__)


class SubprocessError(RuntimeError):
    def __init__(self, message: str, *, transient: bool) -> None:
        super().__init__(message)
        self.transient = transient


@dataclass(slots=True)
class SubprocessResult:
    exit_code: int
    timed_out: bool
    stdout_path: Path
    stderr_path: Path
    output_exceeded: bool = False


_STDERR_POLL_INTERVAL = 5.0
_HEARTBEAT_INTERVAL = 60.0
_OUTPUT_EXCEEDED_EXIT = 125

_NOTABLE_RE = re.compile(
    r"quota|rate.?limit|429|too many requests|overloaded|retrying|"
    r"Operation cancelled|exhausted.*capacity|"
    r"credit balance|insufficient.{0,20}(funds|credits|balance)|"
    r"not logged in|upgrade your subscription|hit your .{0,12}limit",
    re.IGNORECASE,
)


def run_subprocess(  # noqa: PLR0913
    *,
    run_args: str | list[str],
    env: dict[str, str],
    cwd: Path | None = None,
    timeout_seconds: int,
    stdout_path: Path,
    stderr_path: Path,
    log_label: str = "",
    monitor_stderr: bool = True,
    stop_event: threading.Event | None = None,
    max_output_bytes: int | None = None,
    error_line: re.Pattern[str] | None = None,
) -> SubprocessResult:
    with (
        stdout_path.open("w", encoding="utf-8") as stdout_handle,
        stderr_path.open("w", encoding="utf-8") as stderr_handle,
    ):
        process = subprocess.Popen(  # noqa: S603
            run_args,
            env=env,
            cwd=cwd,
            stdin=subprocess.DEVNULL,
            stdout=stdout_handle,
            stderr=stderr_handle,
            text=True,
        )
        start_monotonic = time.monotonic()
        last_output_check = start_monotonic
        last_heartbeat = start_monotonic
        stderr_offset = 0
        stdout_offset = 0

        try:
            while True:
                returncode = process.poll()
                if returncode is not None:
                    return SubprocessResult(
                        exit_code=returncode,
                        timed_out=False,
                        stdout_path=stdout_path,
                        stderr_path=stderr_path,
                    )

                now = time.monotonic()
                elapsed = now - start_monotonic

                if stop_event is not None and stop_event.is_set():
                    _terminate_process(process)
                    return SubprocessResult(
                        exit_code=130,
                        timed_out=False,
                        stdout_path=stdout_path,
                        stderr_path=stderr_path,
                    )

                if elapsed >= timeout_seconds:
                    _terminate_process(process)
                    return SubprocessResult(
                        exit_code=124,
                        timed_out=True,
                        stdout_path=stdout_path,
                        stderr_path=stderr_path,
                    )

                if (
                    max_output_bytes is not None
                    and _file_size(stdout_path) + _file_size(stderr_path) > max_output_bytes
                ):
                    _terminate_process(process)
                    return SubprocessResult(
                        exit_code=_OUTPUT_EXCEEDED_EXIT,
                        timed_out=False,
                        stdout_path=stdout_path,
                        stderr_path=stderr_path,
                        output_exceeded=True,
                    )

                if now - last_output_check >= _STDERR_POLL_INTERVAL:
                    if monitor_stderr:
                        stderr_offset = _check_output(
                            stderr_path,
                            stderr_offset,
                            log_label,
                            "stderr",
                            error_line,
                        )
                    stdout_offset = _check_output(
                        stdout_path,
                        stdout_offset,
                        log_label,
                        "stdout",
                        error_line,
                    )
                    last_output_check = now

                if now - last_heartbeat >= _HEARTBEAT_INTERVAL:
                    logger.info(
                        "%s: still running (%ds elapsed, %ds until timeout,"
                        " stdout=%d bytes, stderr=%d bytes)",
                        log_label,
                        int(elapsed),
                        timeout_seconds - int(elapsed),
                        _file_size(stdout_path),
                        _file_size(stderr_path),
                    )
                    last_heartbeat = now

                time.sleep(0.1)
        except KeyboardInterrupt:
            _terminate_process(process)
            raise


def _file_size(path: Path) -> int:
    try:
        return path.stat().st_size
    except OSError:
        return 0


# CLI transcripts echo the prompt, so only lines matching *error_line* may raise a warning.
def _check_output(  # noqa: PLR0913
    path: Path,
    offset: int,
    label: str,
    stream: str,
    error_line: re.Pattern[str] | None = None,
) -> int:
    try:
        size = path.stat().st_size
    except OSError:
        return offset
    if size <= offset:
        return offset
    try:
        with path.open("r", encoding="utf-8", errors="replace") as fh:
            fh.seek(offset)
            new_text = fh.read()
    except OSError:
        return offset
    for raw_line in new_text.splitlines():
        stripped = raw_line.strip()
        if error_line is not None and not error_line.search(stripped):
            continue
        if stripped and _NOTABLE_RE.search(stripped):
            logger.warning("%s: agent %s: %s", label, stream, stripped)
            break  # one warning per poll interval to avoid log spam
    return size


def _terminate_process(process: subprocess.Popen[str] | subprocess.Popen[bytes]) -> None:
    try:
        process.terminate()
    except OSError:
        return
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        try:
            process.kill()
        except OSError:
            return
        process.wait(timeout=2)
