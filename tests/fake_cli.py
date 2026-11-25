"""Fake `codex exec`, `claude -p` and `agy -p`; answer Coach prompts without a model."""

import json
import os
import re
import sys
import time
from pathlib import Path

SOURCE_SEPARATOR = " — "
USAGE = {"input_tokens": 1200, "cache_read": 300, "cache_write": 40, "output": 90, "reasoning": 25}
# Codex and Antigravity report cached prompt tokens inside their input count.
INCLUSIVE_INPUT = USAGE["input_tokens"] + USAGE["cache_read"]


def executable(directory: Path, vendor: str = "codex") -> Path:
    # The runner recognizes Antigravity by its command name.
    wrapper = directory / ("agy" if vendor == "gemini" else f"fake-{vendor}")
    wrapper.write_text(
        f"#!{sys.executable}\nimport runpy\nrunpy.run_path({str(Path(__file__))!r}, "
        f"init_globals={{'VENDOR': {vendor!r}}}, run_name='__main__')\n"
    )
    wrapper.chmod(0o755)
    return wrapper


REJECTIONS = {
    "zzqx": "This is not an English word or expression.",
    "Guten Morgen": "This is German, not English.",
    "fom": "“fom” could be several English words, such as from or form.",
}
CORRECTIONS = {"definately": "definitely", "belive": "believe"}


def recognition(typed: dict[str, str]) -> dict[str, object]:
    text = typed["text"]
    rejected = next((reason for key, reason in REJECTIONS.items() if key in text), None)
    if rejected:
        return {
            "outcome": "reject",
            "original_span": "",
            "target": "",
            "source": "",
            "source_usable": False,
            "reason": rejected,
        }
    original, _, source = text.partition(SOURCE_SEPARATOR)
    if not source and len(text.split()) >= 4:
        source, original = text, " ".join(text.split()[1:3])
    target = original
    for typo, fixed in CORRECTIONS.items():
        target = target.replace(typo, fixed)
    return {
        "outcome": "accept",
        "original_span": original,
        "target": target,
        "source": source,
        # A misspelled source sentence is faulty surroundings for the target.
        "source_usable": bool(source) and "tomorow" not in source,
        "reason": "",
    }


def judged(typed: dict[str, str], step: str) -> dict[str, object]:
    """The writer's or checker's recognition; markers in the text make the two disagree."""
    text = typed["text"]
    found = recognition(typed)
    if f"{step}-rejects" in text:
        found = {**found, "outcome": "reject", "original_span": "", "target": ""}
        found.update(source="", source_usable=False, reason="This is not an English expression.")
    if step == "check" and "check-target" in text and found["outcome"] == "accept":
        found = {**found, "target": str(found["target"]).split()[0]}
        found["original_span"] = found["target"]
    target = str(found["target"])
    return {**found, "kind": "word" if len(target.split()) == 1 else "phrase"}


def draft(typed: dict[str, str]) -> dict[str, object]:
    found = judged(typed, "write")
    target = str(found["target"])
    if found["outcome"] == "reject":
        return {
            "recognition": found,
            "explanation": "",
            "examples": [],
            "recall_hints": [],
            "usage_contexts": [],
        }
    examples = []
    if found["source_usable"]:
        examples.append(
            {"text": found["source"], "partner_line": "", "attribution": "source"},
        )
    # "leaky-partner" makes the first new example's partner line give the answer away.
    leaky = f"Do you {target}?" if "leaky-partner" in typed["text"] else ""
    examples += [
        {
            "text": f"My friend said {target} at the meeting.",
            "partner_line": leaky,
            "attribution": "generated",
        },
        {"text": f"We heard {target} again on the radio today.", "attribution": "generated"},
        {"text": f"Her teacher wrote {target} in the report.", "attribution": "generated"},
    ]
    first = target.split()[0]
    half = max(1, len(first) // 2)
    terse = "terse" in typed["text"]
    return {
        "recognition": found,
        "explanation": "яд" if terse else f"Target: {target}. A useful way to say it.",
        "examples": [
            {"partner_line": "", **example, "gloss": f"sense {index}"}
            for index, example in enumerate(examples, start=1)
        ],
        "recall_hints": [
            f"{first[:half]}... / morning",
            f"{first[: max(1, half - 1)]}... / garden",
            f"{first[:half]}... / station",
        ],
        "usage_contexts": ["coach test", "fake practice"],
    }


def check(typed: dict[str, str], prompt: str) -> dict[str, object]:
    text = typed["text"]
    shown = json.loads(re.search(r"<draft>\n(.*?)\n</draft>", prompt, re.S)[1])
    examples = shown["examples"]
    generated = [index for index, item in enumerate(examples) if item["attribution"] == "generated"]
    # "unsafe-example" doubts the first new example, "unsafe-new" every new example.
    unsafe = (
        generated if "unsafe-new" in text else generated[:1] if "unsafe-example" in text else []
    )
    return {
        "recognition": judged(typed, "check"),
        "explanation_ok": "unsafe-meaning" not in text,
        "example_ok": [index not in unsafe for index in range(len(examples))],
        "hints_ok": [
            not ("unsafe-hint" in text and index == 0)
            for index in range(len(shown["recall_hints"]))
        ],
        "usage_contexts_ok": [
            not ("unsafe-context" in text and index == 0)
            for index in range(len(shown["usage_contexts"]))
        ],
        "reasons": ["The first new example changes the meaning."] if "unsafe" in text else [],
    }


CODEX_FAILURES = {
    "echo-transcript": None,
    "logged-out": "ERROR: Not logged in. Run `codex login` first.",
    "usage-limit": "ERROR: unexpected status 429 Too Many Requests",
    "subscription-limit": "ERROR: You've hit your usage limit. Visit "
    "https://chatgpt.com/settings/usage or try again at Nov 13th, 2026 4:58 PM.",
    # Codex's --json events are compact JSON lines on stdout.
    "event-limit": json.dumps(
        {"type": "error", "message": "You've hit your usage limit. Try again later."},
        separators=(",", ":"),
    ),
}
CLAUDE_FAILURES = {
    "echo-transcript": None,
    "logged-out": "Invalid API key · Please run /login",
    "usage-limit": "API Error: 429 Too Many Requests",
    "subscription-limit": "You've hit your 5-hour limit · resets 6pm (Europe/Belgrade)",
}
GEMINI_FAILURES = {
    "echo-transcript": None,
    "logged-out": "You are not logged into Antigravity.",
    "usage-limit": "E1115 10:00:00.000000 1 client.go:1] RetryableQuotaError: You have "
    "exhausted your capacity on this model.",
    "subscription-limit": "Individual quota reached. Resets in 103h45m22s",
}


def typed_input(prompt: str) -> dict[str, str]:
    return json.loads(re.search(r"<learner_input>\n(.*?)\n</learner_input>", prompt, re.S)[1])


def fail(vendor: str, text: str, prompt: str, step: str) -> None:
    failures = {"codex": CODEX_FAILURES, "gemini": GEMINI_FAILURES}.get(vendor, CLAUDE_FAILURES)
    if step == "check" and "checker-limit" in text:
        text = "subscription-limit"
    for key, message in failures.items():
        if key not in text:
            continue
        if message is None:
            print(f"user\n{prompt}\nI could not finish.", file=sys.stderr)
        elif message.startswith("{"):
            print(message)
        elif vendor == "codex" or message.startswith(("API Error", "E1115")):
            print(message, file=sys.stderr)
        elif vendor == "gemini":
            print(json.dumps({"status": "ERROR", "response": "", "error": message}))
        else:
            print(
                json.dumps(
                    {"type": "result", "subtype": "success", "is_error": True, "result": message}
                )
            )
        sys.exit(1)


def respond(vendor: str, arguments: list[str], answer_value: object | None) -> None:
    if vendor == "codex":
        output = Path(arguments[arguments.index("--output-last-message") + 1])
        output.write_text("{not json" if answer_value is None else json.dumps(answer_value))
        assert "--json" in arguments, arguments
        usage = {
            "input_tokens": INCLUSIVE_INPUT,
            "cached_input_tokens": USAGE["cache_read"],
            "cache_write_input_tokens": 0,
            "output_tokens": USAGE["output"],
            "reasoning_output_tokens": USAGE["reasoning"],
        }
        print(json.dumps({"type": "turn.started"}))
        print(json.dumps({"type": "turn.completed", "usage": usage}, separators=(",", ":")))
        return
    if vendor == "gemini":
        # news-recap's Antigravity command with the runner's log file.
        assert arguments[-2] == "-p", arguments
        assert {"--dangerously-skip-permissions", "--log-file"} <= set(arguments), arguments
        response = "I could not decide." if answer_value is None else json.dumps(answer_value)
        usage = {
            "input_tokens": INCLUSIVE_INPUT,
            "cache_read_tokens": USAGE["cache_read"],
            "output_tokens": USAGE["output"],
            "total_tokens": INCLUSIVE_INPUT + USAGE["output"],
        }
        print(
            json.dumps(
                {"status": "SUCCESS", "response": f"```json\n{response}\n```", "usage": usage}
            )
        )
        return
    model = arguments[arguments.index("--model") + 1]
    envelope = {
        "type": "result",
        "subtype": "success",
        "is_error": False,
        "result": "done",
        "num_turns": 2,
        "total_cost_usd": 0.01,
        "usage": {
            "input_tokens": USAGE["input_tokens"],
            "cache_read_input_tokens": USAGE["cache_read"],
            "cache_creation_input_tokens": USAGE["cache_write"],
            "output_tokens": USAGE["output"],
            "output_tokens_details": {"thinking_tokens": USAGE["reasoning"]},
        },
        "modelUsage": {model: {"outputTokens": USAGE["output"]}},
    }
    if answer_value is not None:
        envelope["structured_output"] = answer_value
    print(json.dumps(envelope))


def answer(step: str, typed: dict[str, str], prompt: str) -> dict[str, object] | None:
    text = typed["text"]
    if "malformed" in text or (step == "check" and "garbled" in text):
        return None
    if step == "write":
        return draft(typed)
    if step == "check":
        return check(typed, prompt)
    return recognition(typed)


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
        respond(vendor, arguments, {"environment": sorted(os.environ)})
        return
    typed = typed_input(prompt)
    text = typed["text"]
    fail(vendor, text, prompt, step)
    if "slow" in text or (step == "check" and "sluggish" in text):
        print(os.getpid(), file=sys.stderr, flush=True)
        time.sleep(30)
    respond(vendor, arguments, answer(step, typed, prompt))


if __name__ == "__main__":
    main()
