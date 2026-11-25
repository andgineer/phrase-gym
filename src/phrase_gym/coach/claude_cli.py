import json
import re
import shlex
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from phrase_gym.coach.agents.config import (
    _DEFAULT_AGENT_API_KEY_VARS,
    _DEFAULT_CLAUDE_CMD,
    CLAUDE_WRITER,
)
from phrase_gym.coach.cli_steps import (
    STEP_TIMEOUT,
    CliError,
    attempts_root,
    decode_object,
    json_envelope,
    run_step,
    write_step_files,
)
from phrase_gym.coach.usage import claude_usage, record_step

# Claude prints its own errors as these stderr lines or as an error result envelope on stdout.
ERROR_LINE = re.compile(r'^(?:Error|API Error)\b.*$|^.*"is_error":\s*true.*$', re.M)


def template_values(
    attempt: Path,
    schema: dict[str, Any],
    model: str = CLAUDE_WRITER,
) -> dict[str, str | Path]:
    """Only Read exists: allowed inside this attempt; home and .env files are denied by rule."""
    return {
        "model": model,
        "schema": shlex.quote(json.dumps(schema)),
        "read_scope": shlex.quote(f"Read(/{attempt}/**)"),
    }


@dataclass(frozen=True)
class ClaudeCli:
    executable: str = "claude"
    timeout: float = STEP_TIMEOUT
    attempts: Path = field(default_factory=attempts_root)
    model: str = CLAUDE_WRITER

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
        started = time.monotonic()
        try:
            await run_step(
                "Claude",
                self.command_template,
                template_values(attempt, schema, self.model),
                attempt=attempt,
                step=step,
                cwd=attempt,
                timeout=self.timeout,
                api_key_vars=_DEFAULT_AGENT_API_KEY_VARS["claude"],
                error_line=ERROR_LINE,
            )
        finally:
            envelope = json_envelope(_stdout(attempt, step))
            elapsed = time.monotonic() - started
            usage = claude_usage(envelope)
            record_step(attempt, step, cli="claude", flags=self.model, elapsed=elapsed, usage=usage)
        if envelope.get("is_error") or envelope.get("subtype") not in (None, "success"):
            detail = str(envelope.get("result") or envelope.get("subtype"))[:300]
            raise CliError(f"Claude: {detail} (during {step})")
        answer = envelope.get("structured_output")
        if not isinstance(answer, dict):
            raise CliError(f"Claude returned no structured answer during {step}")
        content = json.dumps(answer, ensure_ascii=False)
        (attempt / f"{step}-answer.json").write_text(content, encoding="utf-8")
        return decode_object(content, "Claude", step)


def _stdout(attempt: Path, step: str) -> str:
    try:
        return (attempt / f"{step}-stdout.log").read_text(errors="replace")
    except OSError:
        return ""
