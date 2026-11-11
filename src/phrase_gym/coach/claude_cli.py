import json
import re
import shlex
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from phrase_gym.coach.agents.config import _DEFAULT_AGENT_API_KEY_VARS, _DEFAULT_CLAUDE_CMD
from phrase_gym.coach.cli_steps import (
    STEP_TIMEOUT,
    CliError,
    decode_object,
    run_step,
    write_step_files,
)

# Claude prints its own errors as these stderr lines or as an error result envelope on stdout.
ERROR_LINE = re.compile(r'^(?:Error|API Error)\b.*$|^.*"is_error":\s*true.*$', re.M)


def attempts_root() -> Path:
    return Path(tempfile.gettempdir()).resolve() / "phrase-gym-coach"


def template_values(attempt: Path, schema: dict[str, Any]) -> dict[str, str | Path]:
    """Only Read exists: allowed inside this attempt; home and .env files are denied by rule."""
    return {
        "schema": shlex.quote(json.dumps(schema)),
        "read_scope": shlex.quote(f"Read(/{attempt}/**)"),
    }


@dataclass(frozen=True)
class ClaudeCli:
    executable: str = "claude"
    timeout: float = STEP_TIMEOUT
    attempts: Path = field(default_factory=attempts_root)

    @property
    def command_template(self) -> str:
        return _DEFAULT_CLAUDE_CMD.replace("claude", shlex.quote(self.executable), 1)

    async def ask(
        self,
        attempt: Path,
        step: str,
        prompt: str,
        schema: dict[str, Any],
    ) -> dict[str, Any]:
        attempt = attempt.resolve()
        write_step_files(attempt, step, prompt, schema)
        result = await run_step(
            "Claude",
            self.command_template,
            template_values(attempt, schema),
            attempt=attempt,
            step=step,
            cwd=attempt,
            timeout=self.timeout,
            api_key_vars=_DEFAULT_AGENT_API_KEY_VARS["claude"],
            error_line=ERROR_LINE,
        )
        envelope = _envelope(result.stdout_path.read_text(errors="replace"))
        if envelope.get("is_error") or envelope.get("subtype") not in (None, "success"):
            detail = str(envelope.get("result") or envelope.get("subtype"))[:300]
            raise CliError(f"Claude: {detail} (during {step})")
        answer = envelope.get("structured_output")
        if not isinstance(answer, dict):
            raise CliError(f"Claude returned no structured answer during {step}")
        content = json.dumps(answer, ensure_ascii=False)
        (attempt / f"{step}-answer.json").write_text(content, encoding="utf-8")
        return decode_object(content, "Claude", step)


def _envelope(stdout: str) -> dict[str, Any]:
    try:
        decoded = json.loads(stdout)
    except ValueError:
        return {}
    return decoded if isinstance(decoded, dict) else {}
