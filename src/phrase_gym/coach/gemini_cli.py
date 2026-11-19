import json
import re
import shlex
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from phrase_gym.coach.agents.config import (
    _DEFAULT_AGENT_API_KEY_VARS,
    _DEFAULT_GEMINI_CMD,
    GEMINI_MODEL,
)
from phrase_gym.coach.agents.subprocess import SubprocessResult
from phrase_gym.coach.cli_steps import (
    STEP_TIMEOUT,
    CliError,
    attempts_root,
    decode_object,
    json_envelope,
    run_step,
    write_step_files,
)

# agy's log says "not logged into Antigravity" on every start, so only the answer envelope's
# error and agy's own quota lines count.
ERROR_LINE = re.compile(
    r"^ERROR:.*$|^.*(?:Individual quota reached|RetryableQuotaError|upgrade your subscription).*$",
    re.M,
)
MODEL_REJECTED = re.compile(r"invalid model selection|requires --effort", re.I)
FLASH_MODEL_ID = re.compile(r"gemini-[0-9]+(?:\.[0-9]+)*-flash(?:-[a-z0-9]+)*")
EFFORTS = ("minimal", "low", "medium", "high", "xhigh", "max")
MODELS_LIST_TIMEOUT = 60


def smartest_flash_model(listing: str) -> str | None:
    ids = [line.split("\t", 1)[0].strip() for line in listing.splitlines() if "\t" in line]
    flash = [model for model in ids if FLASH_MODEL_ID.fullmatch(model)]
    return max(flash, key=_flash_rank) if flash else None


def _flash_rank(model: str) -> tuple[tuple[int, ...], bool, int]:
    suffix = model.rsplit("-", 1)[-1]
    # A name without an effort suffix runs at agy's default thinking level.
    effort = EFFORTS.index(suffix if suffix in EFFORTS else "medium")
    version = re.search(r"\d+(?:\.\d+)*", model)
    newest = tuple(int(part) for part in version.group().split(".")) if version else ()
    return newest, "lite" not in model, effort


def _surface_envelope_error(result: SubprocessResult) -> None:
    """Puts the envelope's error first in the log, where failures are classified."""
    try:
        envelope = json_envelope(result.stdout_path.read_text(errors="replace"))
        if error := envelope.get("error"):
            log = result.stderr_path.read_text(errors="replace")
            line = " ".join(str(error).split())
            result.stderr_path.write_text(f"ERROR: {line}\n{log}", encoding="utf-8")
    except OSError:
        return


@dataclass
class GeminiCli:
    executable: str = "agy"
    timeout: float = STEP_TIMEOUT
    attempts: Path = field(default_factory=attempts_root)
    model: str = GEMINI_MODEL

    @property
    def command_template(self) -> str:
        return _DEFAULT_GEMINI_CMD.replace("agy", shlex.quote(self.executable), 1)

    def template_values(self, attempt: Path, step: str) -> dict[str, str | Path]:
        return {
            "model": self.model,
            "log_file": attempt / f"{step}-stderr.log",
            "schema_file": attempt / f"{step}-schema.json",
        }

    async def ask(
        self,
        attempt: Path,
        step: str,
        prompt: str,
        schema: dict[str, Any],
    ) -> dict[str, Any]:
        attempt = attempt.resolve()
        write_step_files(attempt, step, prompt, schema)
        try:
            envelope = await self._run(attempt, step)
        except CliError as error:
            # agy renames its models without notice; a rejected name moves to the listed Flash.
            if not MODEL_REJECTED.search(str(error)):
                raise
            listed = await self._listed_flash(attempt)
            if listed is None or f"--model {listed}" == self.model:
                raise
            self.model = f"--model {listed}"
            envelope = await self._run(attempt, step)
        answer = envelope.get("structured_output")
        if not isinstance(answer, dict):
            raise CliError(f"Gemini returned no structured answer during {step}")
        content = json.dumps(answer, ensure_ascii=False)
        (attempt / f"{step}-answer.json").write_text(content, encoding="utf-8")
        return decode_object(content, "Gemini", step)

    async def _run(self, attempt: Path, step: str) -> dict[str, Any]:
        result = await run_step(
            "Gemini",
            self.command_template,
            self.template_values(attempt, step),
            attempt=attempt,
            step=step,
            cwd=attempt,
            timeout=self.timeout,
            api_key_vars=_DEFAULT_AGENT_API_KEY_VARS["gemini"],
            error_line=ERROR_LINE,
            before_check=_surface_envelope_error,
        )
        envelope = json_envelope(result.stdout_path.read_text(errors="replace"))
        if envelope.get("error") or envelope.get("status") not in (None, "SUCCESS"):
            detail = str(envelope.get("error") or envelope.get("status"))[:300]
            raise CliError(f"Gemini: {detail} (during {step})")
        return envelope

    async def _listed_flash(self, attempt: Path) -> str | None:
        try:
            result = await run_step(
                "Gemini",
                f"{shlex.quote(self.executable)} models",
                {},
                attempt=attempt,
                step="list-models",
                cwd=attempt,
                timeout=min(self.timeout, MODELS_LIST_TIMEOUT),
                api_key_vars=_DEFAULT_AGENT_API_KEY_VARS["gemini"],
                error_line=ERROR_LINE,
            )
        except CliError:
            return None
        return smartest_flash_model(result.stdout_path.read_text(errors="replace"))
