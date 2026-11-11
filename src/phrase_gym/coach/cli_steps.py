import asyncio
import json
import re
from pathlib import Path
from typing import Any

from phrase_gym.coach.agents.ai_agent import _run_agent_cli, agent_failure
from phrase_gym.coach.agents.subprocess import SubprocessError, SubprocessResult

STEP_TIMEOUT = 600
MAX_ANSWER_BYTES = 256 * 1024


class CliError(RuntimeError):
    pass


def write_step_files(attempt: Path, step: str, prompt: str, schema: dict[str, Any]) -> Path:
    attempt.mkdir(parents=True, mode=0o700, exist_ok=True)
    prompt_path = attempt / f"{step}-prompt.md"
    prompt_path.write_text(prompt, encoding="utf-8")
    (attempt / f"{step}-schema.json").write_text(json.dumps(schema), encoding="utf-8")
    return prompt_path


# Both CLIs run one step through the copied news-recap runner; adapters supply template values.
async def run_step(  # noqa: PLR0913
    agent: str,
    command_template: str,
    values: dict[str, str | Path],
    *,
    attempt: Path,
    step: str,
    cwd: Path,
    timeout: float,
    api_key_vars: list[str],
    error_line: re.Pattern[str],
) -> SubprocessResult:
    try:
        result = await asyncio.to_thread(
            _run_agent_cli,
            prompt_file=attempt / f"{step}-prompt.md",
            stdout_path=attempt / f"{step}-stdout.log",
            stderr_path=attempt / f"{step}-stderr.log",
            cwd=cwd,
            timeout_seconds=int(timeout) or 1,
            command_template=command_template,
            values=values,
            api_key_vars=api_key_vars,
            log_label=f"{agent} {step}",
            error_line=error_line,
        )
    except SubprocessError as error:
        raise CliError(f"{agent}: {error} (during {step})") from error
    if failure := agent_failure(result, agent, error_line):
        raise CliError(f"{failure} (during {step})")
    return result


def decode_object(content: str, label: str, step: str) -> dict[str, Any]:
    if len(content.encode()) > MAX_ANSWER_BYTES:
        raise CliError(f"{label} answer exceeded {MAX_ANSWER_BYTES} bytes during {step}")
    try:
        decoded = json.loads(content)
    except ValueError:
        raise CliError(f"{label} returned malformed JSON during {step}") from None
    if not isinstance(decoded, dict):
        raise CliError(f"{label} returned a non-object answer during {step}")
    return decoded
