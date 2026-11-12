import asyncio
import base64
import json
import os
import time
import urllib.error
import urllib.request
from typing import Any
from urllib.parse import urlsplit

from phrase_gym.coach.claude_cli import ClaudeCli
from phrase_gym.coach.codex_cli import CodexCli, coach_workspace
from phrase_gym.coach.preparation import CardPreparer, PreparationError
from phrase_gym.coach.speech import PiperSpeech, configured_speech

MAX_TCP_PORT = 65_535


class CoachApi:
    def __init__(self, base_url: str):
        parsed = urlsplit(base_url)
        try:
            port = parsed.port
        except ValueError:
            port = None
        if (
            parsed.scheme != "http"
            or parsed.hostname not in {"127.0.0.1", "localhost"}
            or parsed.username is not None
            or parsed.password is not None
            or port is None
            or not 1 <= port <= MAX_TCP_PORT
            or parsed.path not in {"", "/"}
            or parsed.query
            or parsed.fragment
        ):
            raise ValueError("Coach requires a loopback Phrase Gym URL")
        self.base_url = f"http://{parsed.hostname}:{port}"

    def post(self, path: str, payload: dict[str, object] | None = None) -> Any:
        body = json.dumps(payload or {}).encode()
        request = urllib.request.Request(  # noqa: S310
            f"{self.base_url}/api/imports/{path}",
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(request, timeout=90) as response:  # noqa: S310
            return json.load(response)


CLI_CHOICES = ("claude", "codex")


def configured_preparer(choice: str) -> CardPreparer:
    if choice == "codex":
        codex = CodexCli(coach_workspace(), os.environ.get("PHRASE_GYM_CODEX", "codex"))
        return CardPreparer(codex, codex.attempts)
    claude = ClaudeCli(os.environ.get("PHRASE_GYM_CLAUDE", "claude"))
    return CardPreparer(claude, claude.attempts)


async def prepare(text: str, package_id: str, preparer: CardPreparer, speech: PiperSpeech) -> dict:
    phrase = await preparer.prepare(text, package_id)
    audio = await speech.synthesize(phrase.text)
    return {
        "result_id": package_id,
        "phrase": phrase.model_dump(mode="json", exclude_none=True),
        "reference": base64.b64encode(audio).decode(),
    }


def refusal(error: urllib.error.HTTPError) -> str:
    try:
        detail = json.loads(error.read() or b"{}").get("detail")
    except (OSError, ValueError, AttributeError):
        detail = None
    if isinstance(detail, list):
        detail = "; ".join(
            f"{'.'.join(str(part) for part in item.get('loc', []))}: {item.get('msg')}"
            for item in detail
            if isinstance(item, dict)
        )
    return detail if isinstance(detail, str) and detail else str(error)


def process_once(api: CoachApi, preparer: CardPreparer, speech: PiperSpeech) -> bool:
    work = api.post("coach/next")
    if work is None:
        return False
    package_id = work["package_id"]
    details = ""
    retry_helps = True
    try:
        api.post(
            f"{package_id}/result",
            asyncio.run(prepare(work["text"], package_id, preparer, speech)),
        )
        return True
    except urllib.error.HTTPError as error:
        reason = refusal(error)[:1000]
    except (OSError, ValueError, RuntimeError) as error:
        reason = str(error)[:1000]
        if isinstance(error, PreparationError):
            retry_helps = error.retry_helps
            details = f" (details: {error.attempt})" if error.attempt else ""
    print(f"Coach could not prepare “{work['text']}”: {reason}{details}", flush=True)
    api.post(f"{package_id}/failure", {"error": reason, "retry_helps": retry_helps})
    return True


def run_coach(url: str, *, once: bool = False, cli: str = "claude") -> None:
    api = CoachApi(url)
    preparer = configured_preparer(cli)
    speech = configured_speech()
    while True:
        try:
            found = process_once(api, preparer, speech)
        except (OSError, urllib.error.HTTPError) as error:
            print(f"Coach cannot reach Phrase Gym: {error}", flush=True)
            found = False
        if once:
            return
        if not found:
            time.sleep(2)
