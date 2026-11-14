import json
import os
import re
import shlex
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from platformdirs import user_cache_path

from phrase_gym.coach.agents.config import _DEFAULT_AGENT_API_KEY_VARS, _DEFAULT_CODEX_CMD
from phrase_gym.coach.cli_steps import (
    STEP_TIMEOUT,
    CliError,
    decode_object,
    run_step,
    write_step_files,
)

PROFILE = "phrase-gym-coach"
# The transcript echoes the prompt and learner text, so only Codex's own error lines count.
ERROR_LINE = re.compile(r"^ERROR:.*$", re.M)


def coach_workspace() -> Path:
    if configured := os.environ.get("PHRASE_GYM_COACH_WORKSPACE"):
        return Path(configured)
    checkout = Path(__file__).resolve().parents[3]
    if (checkout / "pyproject.toml").is_file() and (checkout / "src" / "phrase_gym").is_dir():
        return checkout
    return user_cache_path("phrase-gym-coach")


def _toml(value: str) -> str:
    return json.dumps(value)


def permission_profile(workspace: Path, attempt: Path) -> str:
    """Reads stay inside the system minimum and this attempt; .env files are denied outright."""
    entries = {
        ":minimal": "read",
        # The minimal set lets commands write to /tmp; this read-only entry withdraws that.
        "/tmp": "read",  # noqa: S108 - a restriction, not a temporary file
        str(attempt): "read",
        str(workspace / ".env"): "none",
        str(workspace / "**" / ".env"): "none",
    }
    body = ", ".join(f"{_toml(path)}={_toml(access)}" for path, access in entries.items())
    return f"permissions.{PROFILE}.filesystem={{{body}}}"


@dataclass(frozen=True)
class CodexCli:
    workspace: Path
    executable: str = "codex"
    timeout: float = STEP_TIMEOUT

    @property
    def attempts(self) -> Path:
        return self.workspace / ".phrase-gym-coach" / "attempts"

    @property
    def command_template(self) -> str:
        return _DEFAULT_CODEX_CMD.replace("codex", shlex.quote(self.executable), 1)

    def template_values(self, attempt: Path, step: str) -> dict[str, str | Path]:
        return {
            "default_permissions": shlex.quote(f'default_permissions="{PROFILE}"'),
            "permission_profile": shlex.quote(permission_profile(self.workspace, attempt)),
            "schema_file": attempt / f"{step}-schema.json",
            "answer_file": attempt / f"{step}-answer.json",
            "workspace": self.workspace,
        }

    async def ask(
        self,
        attempt: Path,
        step: str,
        prompt: str,
        schema: dict[str, Any],
    ) -> dict[str, Any]:
        write_step_files(attempt, step, prompt, schema)
        answer = attempt / f"{step}-answer.json"
        answer.unlink(missing_ok=True)
        await run_step(
            "Codex",
            self.command_template,
            self.template_values(attempt, step),
            attempt=attempt,
            step=step,
            cwd=self.workspace,
            timeout=self.timeout,
            api_key_vars=_DEFAULT_AGENT_API_KEY_VARS["codex"],
            error_line=ERROR_LINE,
        )
        content = answer.read_text(errors="replace") if answer.is_file() else ""
        if not content.strip():
            raise CliError(f"Codex returned no answer (during {step})")
        return decode_object(content, "Codex", step)
