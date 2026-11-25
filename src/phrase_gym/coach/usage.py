import json
import re
from pathlib import Path
from typing import Any

MODEL_FLAG = re.compile(r"--model\s+(\S+)")
EFFORT_FLAG = re.compile(r"(?:--effort\s+|model_reasoning_effort=)(\w+)")
# Antigravity encodes the effort in the model name, such as gemini-3.8-flash-low.
NAMED_EFFORT = re.compile(r"-(minimal|low|medium|high|xhigh|max)$")
# news-recap's reading of Codex's plain-transcript summary: "tokens used\n12,033".
TOKENS_USED = re.compile(r"tokens\s+used\s*\n\s*([\d,]+)", re.IGNORECASE)
USAGE_SUFFIX = "-usage.json"


def model_and_effort(flags: str) -> dict[str, str | None]:
    model = MODEL_FLAG.search(flags)
    effort = EFFORT_FLAG.search(flags)
    named = NAMED_EFFORT.search(model[1]) if model else None
    return {
        "model": model[1] if model else None,
        "effort": effort[1] if effort else named[1] if named else None,
    }


def _counts(values: dict[str, Any]) -> dict[str, int]:
    return {key: value for key, value in values.items() if isinstance(value, int)}


def _read(path: Path) -> str:
    try:
        return path.read_text(errors="replace")
    except OSError:
        return ""


def claude_usage(envelope: dict[str, Any]) -> dict[str, Any]:
    usage = envelope.get("usage") if isinstance(envelope.get("usage"), dict) else {}
    details = usage.get("output_tokens_details")
    details = details if isinstance(details, dict) else {}
    models = envelope.get("modelUsage")
    found: dict[str, Any] = _counts(
        {
            "input_tokens": usage.get("input_tokens"),
            "cache_read_tokens": usage.get("cache_read_input_tokens"),
            "cache_write_tokens": usage.get("cache_creation_input_tokens"),
            "output_tokens": usage.get("output_tokens"),
            "reasoning_tokens": details.get("thinking_tokens"),
            "turns": envelope.get("num_turns"),
        },
    )
    if isinstance(models, dict) and models:
        found["reported_models"] = sorted(models)
    if isinstance(envelope.get("total_cost_usd"), int | float):
        found["list_price_usd"] = envelope["total_cost_usd"]
    return found


def codex_usage(stdout: Path, stderr: Path) -> dict[str, int]:
    """Usage from `codex exec --json` turn events, or the plain transcript's total."""
    totals: dict[str, int] = {}
    names = {
        "input_tokens": "input_tokens",
        "cached_input_tokens": "cache_read_tokens",
        "cache_write_input_tokens": "cache_write_tokens",
        "output_tokens": "output_tokens",
        "reasoning_output_tokens": "reasoning_tokens",
    }
    for line in _read(stdout).splitlines():
        if not line.startswith('{"type":"turn.completed"'):
            continue
        try:
            usage = json.loads(line).get("usage") or {}
        except ValueError:
            continue
        totals["turns"] = totals.get("turns", 0) + 1
        for source, name in names.items():
            if isinstance(usage.get(source), int):
                totals[name] = totals.get(name, 0) + usage[source]
    if not totals and (match := TOKENS_USED.search(_read(stderr))):
        totals["total_tokens"] = int(match[1].replace(",", ""))
    return totals


def gemini_usage(usage: dict[str, int] | None) -> dict[str, int]:
    usage = usage or {}
    return _counts(
        {
            "input_tokens": usage.get("input_tokens"),
            "cache_read_tokens": usage.get("cache_read_tokens"),
            "output_tokens": usage.get("output_tokens"),
            "reasoning_tokens": usage.get("thinking_tokens"),
            "total_tokens": usage.get("total_tokens"),
        },
    )


def record_step(  # noqa: PLR0913
    attempt: Path,
    step: str,
    *,
    cli: str,
    flags: str,
    elapsed: float,
    usage: dict[str, Any],
) -> None:
    """One CLI step's requested model and effort, its time and whatever usage the CLI reported."""
    record = {
        "step": step,
        "cli": cli,
        **model_and_effort(flags),
        "elapsed_seconds": round(elapsed, 3),
        "usage": usage,
    }
    attempt.mkdir(parents=True, mode=0o700, exist_ok=True)
    (attempt / f"{step}{USAGE_SUFFIX}").write_text(json.dumps(record), encoding="utf-8")


def summarize(attempt: Path, request_id: str, outcome: str) -> Path:
    """A small per-request usage file that outlives the attempt trail deleted after success."""
    steps = []
    for path in sorted(attempt.glob(f"*{USAGE_SUFFIX}"), key=lambda found: found.stat().st_mtime):
        try:
            steps.append(json.loads(path.read_text(encoding="utf-8")))
        except (OSError, ValueError):
            continue
    folder = attempt.parent / "usage"
    folder.mkdir(parents=True, mode=0o700, exist_ok=True)
    summary = folder / f"{attempt.name}.json"
    content = {"request_id": request_id, "outcome": outcome, "steps": steps}
    summary.write_text(json.dumps(content, ensure_ascii=False, indent=2), encoding="utf-8")
    summary.chmod(0o600)
    return summary
