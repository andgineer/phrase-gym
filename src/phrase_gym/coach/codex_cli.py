import json
import os
import re
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from platformdirs import user_cache_path

from phrase_gym.coach.bounded_process import OutputLimitError, run_to_files

CODEX_TIMEOUT = 600.0
MAX_ANSWER_BYTES = 256 * 1024
MAX_LOG_BYTES = 8 * 1024 * 1024
PROFILE = "phrase-gym-coach"
# Only the subscription login in the normal Codex home is used; API keys never reach the child.
INHERITED_ENVIRONMENT = (
    "HOME",
    "USER",
    "LOGNAME",
    "PATH",
    "SHELL",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "CODEX_HOME",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "NO_PROXY",
)
NOT_LOGGED_IN = re.compile(r"\bnot logged in\b|\bcodex login\b|\bunauthorized\b", re.I)
# A bare 429 also matches process and trace ids in CLI logs.
USAGE_LIMIT = re.compile(
    r"\busage limit\b|\bquota\b|\brate.?limit\b|\btoo many requests\b"
    r"|\b(status|code|http)\b\W{0,10}429\b",
    re.I,
)
RESET_AT = re.compile(r"try again at ([^.\n]{1,60})")
# The transcript echoes the prompt and learner text, so only Codex's own error lines count.
ERROR_LINE = re.compile(r"^ERROR:.*$", re.M)


class CliError(RuntimeError):
    pass


def coach_workspace() -> Path:
    if configured := os.environ.get("PHRASE_GYM_COACH_WORKSPACE"):
        return Path(configured)
    checkout = Path(__file__).resolve().parents[3]
    if (checkout / "pyproject.toml").is_file() and (checkout / "src" / "phrase_gym").is_dir():
        return checkout
    return user_cache_path("phrase-gym-coach")


def child_environment() -> dict[str, str]:
    inherited = {name: os.environ[name] for name in INHERITED_ENVIRONMENT if name in os.environ}
    return {**inherited, "TERM": "dumb", "NO_COLOR": "1"}


def _toml(value: str) -> str:
    return json.dumps(value)


def permission_profile(workspace: Path, attempt: Path) -> str:
    """Reads stay inside the system minimum and this attempt; .env files are denied outright."""
    entries = {
        ":minimal": "read",
        str(attempt): "read",
        str(workspace / ".env"): "none",
        str(workspace / "**" / ".env"): "none",
    }
    body = ", ".join(f"{_toml(path)}={_toml(access)}" for path, access in entries.items())
    return f"permissions.{PROFILE}.filesystem={{{body}}}"


def failure_reason(log: str) -> str | None:
    text = "\n".join(ERROR_LINE.findall(log))
    if NOT_LOGGED_IN.search(text):
        return "Codex is not logged in; run `codex login` in a terminal, then submit the text again"
    if USAGE_LIMIT.search(text):
        reset = RESET_AT.search(text)
        when = f"after {reset[1]}" if reset else "later"
        return f"Codex reported a usage limit; submit the text again {when}"
    return None


@dataclass(frozen=True)
class CodexCli:
    workspace: Path
    executable: str = "codex"
    timeout: float = CODEX_TIMEOUT

    def arguments(self, attempt: Path, step: str) -> list[str]:
        return [
            "exec",
            "--skip-git-repo-check",
            "--ephemeral",
            "--color",
            "never",
            "-c",
            f'default_permissions="{PROFILE}"',
            "-c",
            permission_profile(self.workspace, attempt),
            "-c",
            "project_doc_max_bytes=0",
            "--output-schema",
            str(attempt / f"{step}-schema.json"),
            "--output-last-message",
            str(attempt / f"{step}-answer.json"),
            "--cd",
            str(self.workspace),
            f"Read your task from {attempt / f'{step}-prompt.md'} and execute it.",
        ]

    async def ask(
        self,
        attempt: Path,
        step: str,
        prompt: str,
        schema: dict[str, Any],
    ) -> dict[str, Any]:
        environment = child_environment()
        program = shutil.which(self.executable, path=environment.get("PATH"))
        if program is None:
            raise CliError("Codex CLI was not found on PATH; install it and run `codex login`")
        attempt.mkdir(parents=True, mode=0o700, exist_ok=True)
        (attempt / f"{step}-prompt.md").write_text(prompt, encoding="utf-8")
        (attempt / f"{step}-schema.json").write_text(json.dumps(schema), encoding="utf-8")
        answer = attempt / f"{step}-answer.json"
        answer.unlink(missing_ok=True)
        logs = (attempt / f"{step}-stdout.log", attempt / f"{step}-stderr.log")
        try:
            returncode = await run_to_files(
                Path(program),
                self.arguments(attempt, step),
                cwd=self.workspace,
                environment=environment,
                timeout=self.timeout,
                output=logs,
                limit=MAX_LOG_BYTES,
            )
        except TimeoutError:
            raise CliError(f"Codex did not finish {step} within {self.timeout:.0f} s") from None
        except OutputLimitError:
            raise CliError(f"Codex exceeded the output limit during {step}") from None
        log = "".join(path.read_text(errors="replace") for path in reversed(logs))
        if returncode:
            reason = failure_reason(log) or f"Codex exited with code {returncode}"
            raise CliError(f"{reason} (during {step})")
        return self._answer(answer, step, log)

    @staticmethod
    def _answer(path: Path, step: str, log: str) -> dict[str, Any]:
        content = path.read_bytes() if path.is_file() else b""
        if not content.strip():
            reason = failure_reason(log) or "Codex returned no answer"
            raise CliError(f"{reason} (during {step})")
        if len(content) > MAX_ANSWER_BYTES:
            raise CliError(f"Codex answer exceeded {MAX_ANSWER_BYTES} bytes during {step}")
        try:
            decoded = json.loads(content)
        except ValueError:
            raise CliError(f"Codex returned malformed JSON during {step}") from None
        if not isinstance(decoded, dict):
            raise CliError(f"Codex returned a non-object answer during {step}")
        return decoded
