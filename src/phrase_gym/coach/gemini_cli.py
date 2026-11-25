import asyncio
import shlex
import threading
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from phrase_gym.coach.agents.ai_agent import _run_agent_cli
from phrase_gym.coach.agents.antigravity import launch_agy, paced
from phrase_gym.coach.agents.antigravity_errors import agy_failure
from phrase_gym.coach.agents.config import (
    _DEFAULT_AGENT_API_KEY_VARS,
    _DEFAULT_GEMINI_CMD,
    AGENT_LAUNCH_DELAY,
    GEMINI_MODEL,
)
from phrase_gym.coach.agents.subprocess import SubprocessError, SubprocessResult
from phrase_gym.coach.cli_steps import (
    STEP_TIMEOUT,
    CliError,
    attempts_root,
    decode_object,
    write_step_files,
)


def answer_object(response: str, step: str) -> dict[str, Any]:
    # The free route answers in prose mode, so a fenced or introduced JSON object is accepted.
    start, end = response.find("{"), response.rfind("}")
    if start < 0 or end < start:
        raise CliError(f"Gemini returned no JSON object during {step}")
    return decode_object(response[start : end + 1], "Gemini", step)


@dataclass
class GeminiCli:
    """Gemini free through Antigravity's `agy`, run the way news-recap runs it."""

    executable: str = "agy"
    timeout: float = STEP_TIMEOUT
    attempts: Path = field(default_factory=attempts_root)
    model: str = GEMINI_MODEL
    launch_delay: float = AGENT_LAUNCH_DELAY["gemini"]

    @property
    def command_template(self) -> str:
        return _DEFAULT_GEMINI_CMD.replace("agy", shlex.quote(self.executable), 1)

    async def ask(
        self,
        attempt: Path,
        step: str,
        prompt: str,
        schema: dict[str, Any],
    ) -> dict[str, Any]:
        attempt = attempt.resolve()
        # The prompt states the answer shape; the schema file stays with the attempt for review.
        write_step_files(attempt, step, prompt, schema)
        stop_event = threading.Event()
        try:
            result = await asyncio.to_thread(self._launch, attempt, step, stop_event)
        except asyncio.CancelledError:
            # The worker thread outlives the cancelled await; the event makes it stop the child.
            stop_event.set()
            raise
        if failure := agy_failure(result, "Gemini"):
            raise CliError(f"{failure} (during {step})")
        response = result.stdout_path.read_text(errors="replace")
        answer = answer_object(response, step)
        (attempt / f"{step}-answer.json").write_text(response, encoding="utf-8")
        return answer

    def _launch(self, attempt: Path, step: str, stop_event: threading.Event) -> SubprocessResult:
        def launch(model: str) -> SubprocessResult:
            def run() -> SubprocessResult:
                return _run_agent_cli(
                    prompt_file=attempt / f"{step}-prompt.md",
                    stdout_path=attempt / f"{step}-stdout.log",
                    stderr_path=attempt / f"{step}-stderr.log",
                    cwd=attempt,
                    timeout_seconds=int(self.timeout) or 1,
                    command_template=self.command_template,
                    values={"model": model},
                    api_key_vars=_DEFAULT_AGENT_API_KEY_VARS["gemini"],
                    log_label=f"Gemini {step}",
                    stop_event=stop_event,
                )

            try:
                return paced(run, self.launch_delay, stop_event)
            except SubprocessError as error:
                raise CliError(f"Gemini: {error} (during {step})") from error

        result, _ = launch_agy(launch, self.model, self.command_template, f"Gemini {step}")
        return result
