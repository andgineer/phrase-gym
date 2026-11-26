"""Fake `codex exec`, `claude -p` and `agy -p`; answer Coach prompts without a model."""

import json
import os
import re
import sys
import time
from pathlib import Path

SOURCE_SEPARATOR = " — "


def executable(directory: Path, vendor: str = "codex") -> Path:
    # The runner recognizes Antigravity by its command name.
    wrapper = directory / ("agy" if vendor == "gemini" else f"fake-{vendor}")
    wrapper.write_text(
        f"#!{sys.executable}\nimport runpy\nrunpy.run_path({str(Path(__file__))!r}, "
        f"init_globals={{'VENDOR': {vendor!r}}}, run_name='__main__')\n"
    )
    wrapper.chmod(0o755)
    return wrapper


# The fake writes each layer in the language the prompt names, as a model would.
EXAMPLES = {"German": "Das ist {}."}
MEANING_WORDS = {"Russian": ("значение", "Цель"), "German": ("Bedeutung", "Ziel")}


def card(
    typed: str, step: str, layers: tuple[str, str] = ("English", "English")
) -> dict[str, object]:
    learning, meaning = layers
    sense, label = MEANING_WORDS.get(meaning, ("sense", "Target"))
    target, _, source = typed.partition(SOURCE_SEPARATOR)
    word = len(target.split()) == 1
    first = target.split()[0]
    half = max(1, len(first) // 2)
    leaky = (
        "stubborn" in typed
        or ("leaky" in typed and step == "prepare")
        or ("sticky" in typed and step in ("prepare", "revise"))
    )
    cues = [
        {"situation": f"Context {index}: __________ here. (={sense} {index})", "partner_line": None}
        for index in range(1, 4 if word else 3)
    ]
    return {
        "outcome": "ready",
        "reason": "",
        "kind": "word" if word else "phrase",
        "target": target,
        "card": {
            "text": source
            or (EXAMPLES.get(learning, "She felt {} today.").format(target) if word else target),
            "explanation": f"{label}: {target}.",
            "cues": cues,
            "recall_hints": [
                target if leaky else f"{first[:half]}... / one",
                f"{first[: max(1, half - 1)]}... / two",
            ],
            "usage_contexts": ["coach test"],
            "detailed_explanation": None,
        },
    }


REJECTIONS = {
    "zzqx": {
        "English": "This is not an English word or expression.",
        "Russian": "Это не английское слово и не выражение.",
    },
    "Guten Morgen": {
        "English": "This is German, not English.",
        "Russian": "Это немецкий текст, а не английский.",
    },
    "I am very tired": {
        "English": "This is English, not German.",
        "Russian": "Это английский текст, а не немецкий.",
    },
    "fom": {
        "English": "“fom” could be several English words, such as from or form.",
        "Russian": "«fom» может быть разными английскими словами, например from или form.",
    },
}
CORRECTIONS = {"definately": "definitely", "belive": "believe", "die tür": "die Tür"}
INFLECTIONS = {"come up with": "came up with"}
LONGEST_SENTENCE = 200


def recognition(typed: dict[str, str]) -> dict[str, object]:
    text = typed["text"]
    # Each fake rejection judges text that is not in the requested learning language.
    foreign = {"Guten Morgen": "English", "I am very tired": "German"}
    rejected = next(
        (
            reason
            for key, reason in REJECTIONS.items()
            if key in text
            and foreign.get(key, typed["learning_language"]) == typed["learning_language"]
        ),
        None,
    )
    if rejected:
        return {
            "outcome": "reject",
            "original_span": "",
            "target": "",
            "source": "",
            "source_span": "",
            "source_usable": False,
            "reason": rejected[typed["native_language"]],
        }
    original, _, source = text.partition(SOURCE_SEPARATOR)
    # A short sentence offers a span; longer text stays one target that no card can hold.
    if not source and len(text.split()) >= 4 and len(text) <= LONGEST_SENTENCE:
        source, original = text, " ".join(text.split()[1:3])
    target = original
    for typo, fixed in CORRECTIONS.items():
        target = target.replace(typo, fixed)
    # A misspelled source sentence is faulty surroundings for the target.
    usable = bool(source) and "tomorow" not in source
    inflected = INFLECTIONS.get(target, target)
    return {
        "outcome": "accept",
        "original_span": original,
        "target": target,
        "source": source,
        "source_span": (inflected if inflected in source else target) if usable else "",
        "source_usable": usable,
        "reason": "",
    }


def answer(typed: str, step: str, layers: tuple[str, str]) -> dict[str, object]:
    if step.startswith("review"):
        findings = []
        # "leaky" is fixed by the first revision; "picky" and "stubborn" keep drawing findings;
        # "sticky" keeps its leak through the first revision while the second review stays silent.
        first_only = ("leaky", "sticky", "giveup")
        first = step == "review" and any(word in typed for word in first_only)
        if first or "stubborn" in typed or "picky" in typed:
            findings = [
                {
                    "field": "recall_hints.0",
                    "problem": "The hint gives the whole target.",
                    "correction": "Use a partial form.",
                }
            ]
        return {"findings": findings}
    if "insufficient" in typed or ("giveup" in typed and step.startswith("revise")):
        return {
            "outcome": "insufficient",
            "reason": "No English word or expression was found in the input.",
            "kind": "phrase",
            "target": "",
            "card": None,
        }
    return card(typed, step, layers)


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
GEMINI_FAILURES = {
    "echo-transcript": None,
    "logged-out": "You are not logged into Antigravity.",
    "usage-limit": "E1115 10:00:00.000000 1 client.go:1] RetryableQuotaError: You have "
    "exhausted your capacity on this model.",
    "subscription-limit": "Individual quota reached. Resets in 103h45m22s",
}


def typed_input(prompt: str) -> str | dict[str, str]:
    return json.loads(re.search(r"<learner_input>\n(.*?)\n</learner_input>", prompt, re.S)[1])


def fail(vendor: str, typed: str, prompt: str) -> None:
    failures = {"codex": CODEX_FAILURES, "gemini": GEMINI_FAILURES}.get(vendor, CLAUDE_FAILURES)
    for key, message in failures.items():
        if key not in typed:
            continue
        if message is None:
            print(f"user\n{prompt}\nI could not finish.", file=sys.stderr)
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


def respond(vendor: str, arguments: list[str], answer_value: object | None, step: str) -> None:
    if vendor == "codex":
        output = Path(arguments[arguments.index("--output-last-message") + 1])
        output.write_text("{not json" if answer_value is None else json.dumps(answer_value))
        print(output.read_text())
        return
    if vendor == "gemini":
        # news-recap's Antigravity command with the runner's log file.
        assert arguments[-2] == "-p", arguments
        assert {"--dangerously-skip-permissions", "--log-file"} <= set(arguments), arguments
        response = "I could not decide." if answer_value is None else json.dumps(answer_value)
        print(json.dumps({"status": "SUCCESS", "response": f"```json\n{response}\n```"}))
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
    text = typed["text"] if isinstance(typed, dict) else typed
    fail(vendor, text, prompt)
    if "slow" in text:
        print(os.getpid(), file=sys.stderr, flush=True)
        time.sleep(30)
    if "malformed" in text:
        value = None
    elif isinstance(typed, dict):
        value = recognition(typed)
    else:
        learning = re.search(r"Learning language: (\w+),", prompt)
        meaning = re.search(r"Meaning-layer language: (\w+)\.", prompt)
        layers = (learning[1] if learning else "English", meaning[1] if meaning else "English")
        value = answer(typed, step, layers)
    respond(vendor, arguments, value, step)


if __name__ == "__main__":
    main()
