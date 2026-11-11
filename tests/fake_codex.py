"""Stands in for `codex exec`: answers Coach prompts from the typed text without a model."""

import json
import os
import re
import sys
import time
from pathlib import Path

SOURCE_SEPARATOR = " — "


def executable(directory: Path) -> Path:
    wrapper = directory / "fake-codex"
    wrapper.write_text(
        f"#!{sys.executable}\nimport runpy\nrunpy.run_path({str(Path(__file__))!r}, "
        "run_name='__main__')\n"
    )
    wrapper.chmod(0o755)
    return wrapper


def card(typed: str, step: str) -> dict[str, object]:
    target, _, source = typed.partition(SOURCE_SEPARATOR)
    word = len(target.split()) == 1
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
                target if leaky else f"{target[:3]}... / one",
                f"{target[:4]}... / two",
            ],
            "usage_contexts": ["coach test"],
            "accepted_variants": [],
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


def main() -> None:
    arguments = sys.argv[1:]
    output = Path(arguments[arguments.index("--output-last-message") + 1])
    bootstrap = re.fullmatch(r"Read your task from (.+) and execute it\.", arguments[-1])
    assert bootstrap, arguments[-1]
    prompt_path = Path(bootstrap[1])
    step = prompt_path.name.removesuffix("-prompt.md")
    prompt = prompt_path.read_text()
    if prompt == "ECHO-ENVIRONMENT":
        output.write_text(json.dumps({"environment": sorted(os.environ)}))
        return
    typed = json.loads(re.search(r"<learner_input>\n(.*?)\n</learner_input>", prompt, re.S)[1])
    if "usage-limit" in typed:
        print("ERROR: unexpected status 429 Too Many Requests", file=sys.stderr)
        sys.exit(1)
    if "subscription-limit" in typed:
        print(
            "ERROR: You've hit your usage limit. Visit https://chatgpt.com/settings/usage "
            "or try again at Nov 13th, 2026 4:58 PM.",
            file=sys.stderr,
        )
        sys.exit(1)
    if "slow" in typed:
        time.sleep(30)
    if "malformed" in typed:
        output.write_text("{not json")
        return
    print(f"fake codex answered {step}")
    output.write_text(json.dumps(answer(typed, step)))


if __name__ == "__main__":
    main()
