"""Stands in for `codex exec` or `claude -p`: answers Coach prompts from typed text, no model."""

import json
import os
import re
import sys
import time
from pathlib import Path

SOURCE_SEPARATOR = " — "


def executable(directory: Path, vendor: str = "codex") -> Path:
    wrapper = directory / f"fake-{vendor}"
    wrapper.write_text(
        f"#!{sys.executable}\nimport runpy\nrunpy.run_path({str(Path(__file__))!r}, "
        f"init_globals={{'VENDOR': {vendor!r}}}, run_name='__main__')\n"
    )
    wrapper.chmod(0o755)
    return wrapper


def card(typed: str, step: str) -> dict[str, object]:
    target, _, source = typed.partition(SOURCE_SEPARATOR)
    word = len(target.split()) == 1
    first = target.split()[0]
    half = max(1, len(first) // 2)
    leaky = "stubborn" in typed or ("leaky" in typed and step == "prepare")
    cues = [
        {"situation": f"Context {index}: __________ here. (=sense {index})", "partner_line": None}
        for index in range(1, 4 if word else 3)
    ]
    return {
        "outcome": "ready",
        "reason": "",
        "kind": "word" if word else "phrase",
        "target": target,
        "card": {
            "text": source or (f"She felt {target} today." if word else target),
            "explanation": f"Target: {target}. A useful way to say it.",
            "cues": cues,
            "recall_hints": [
                target if leaky else f"{first[:half]}... / one",
                f"{first[: max(1, half - 1)]}... / two",
            ],
            "usage_contexts": ["coach test"],
            "detailed_explanation": None,
        },
    }


def answer(typed: str, step: str) -> dict[str, object]:
    if step == "review":
        findings = []
        if "leaky" in typed or "stubborn" in typed:
            findings = [
                {
                    "field": "recall_hints.0",
                    "problem": "The hint gives the whole target.",
                    "correction": "Use a partial form.",
                }
            ]
        return {"findings": findings}
    if "insufficient" in typed:
        return {
            "outcome": "insufficient",
            "reason": "No English word or expression was found in the input.",
            "kind": "phrase",
            "target": "",
            "card": None,
        }
    return card(typed, step)


CODEX_FAILURES = {
    "echo-transcript": None,
    "logged-out": "ERROR: Not logged in. Run `codex login` first.",
    "usage-limit": "ERROR: unexpected status 429 Too Many Requests",
    "subscription-limit": "ERROR: You've hit your usage limit. Visit "
    "https://chatgpt.com/settings/usage or try again at Nov 13th, 2026 4:58 PM.",
}
CLAUDE_FAILURES = {
    "echo-transcript": None,
    "logged-out": "Invalid API key · Please run /login",
    "usage-limit": "API Error: 429 Too Many Requests",
    "subscription-limit": "You've hit your 5-hour limit · resets 6pm (Europe/Belgrade)",
}


def typed_input(prompt: str) -> str:
    return json.loads(re.search(r"<learner_input>\n(.*?)\n</learner_input>", prompt, re.S)[1])


def fail(vendor: str, typed: str, prompt: str) -> None:
    failures = CODEX_FAILURES if vendor == "codex" else CLAUDE_FAILURES
    for key, message in failures.items():
        if key not in typed:
            continue
        if message is None:
            print(f"user\n{prompt}\nI could not finish.", file=sys.stderr)
        elif vendor == "codex" or message.startswith("API Error"):
            print(message, file=sys.stderr)
        else:
            print(
                json.dumps(
                    {"type": "result", "subtype": "success", "is_error": True, "result": message}
                )
            )
        sys.exit(1)


def respond(vendor: str, arguments: list[str], answer_value: object | None, step: str) -> None:
    if vendor == "codex":
        output = Path(arguments[arguments.index("--output-last-message") + 1])
        output.write_text("{not json" if answer_value is None else json.dumps(answer_value))
        print(output.read_text())
        return
    envelope = {"type": "result", "subtype": "success", "is_error": False, "result": "done"}
    if answer_value is not None:
        envelope["structured_output"] = answer_value
    print(json.dumps(envelope))


def main() -> None:
    # runpy replaces argv[0], so the wrapper passes the vendor it imitates as a global.
    vendor = globals().get("VENDOR", "codex")
    arguments = sys.argv[1:]
    bootstrap = re.fullmatch(r"Read your task from (.+) and execute it\.", arguments[-1])
    assert bootstrap, arguments[-1]
    prompt_path = Path(bootstrap[1])
    step = prompt_path.name.removesuffix("-prompt.md")
    prompt = prompt_path.read_text()
    if prompt == "ECHO-ENVIRONMENT":
        respond(vendor, arguments, {"environment": sorted(os.environ)}, step)
        return
    typed = typed_input(prompt)
    fail(vendor, typed, prompt)
    if "slow" in typed:
        print(os.getpid(), file=sys.stderr, flush=True)
        time.sleep(30)
    respond(vendor, arguments, None if "malformed" in typed else answer(typed, step), step)


if __name__ == "__main__":
    main()
