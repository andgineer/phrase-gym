import asyncio
import base64
import json
import os
import time
import urllib.error
import urllib.request
from typing import Any, Protocol
from urllib.parse import urlsplit

from phrase_gym.coach.claude_cli import ClaudeCli
from phrase_gym.coach.codex_cli import CodexCli, coach_workspace
from phrase_gym.coach.gemini_cli import GeminiCli
from phrase_gym.coach.messages import message
from phrase_gym.coach.preparation import CardPreparer, PreparationError, PreparedPhrase
from phrase_gym.coach.recognition import Recognizer
from phrase_gym.coach.speech import MissingVoiceError, PiperSpeech, configured_speech
from phrase_gym.server.coach_contracts import BasicCard, CoachWork

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


CLI_CHOICES = ("claude", "codex", "gemini")


class Preparer(Protocol):
    async def prepare(self, work: CoachWork) -> PreparedPhrase | BasicCard: ...


def configured_preparer(choice: str) -> Preparer:
    if choice == "codex":
        codex = CodexCli(coach_workspace(), os.environ.get("PHRASE_GYM_CODEX", "codex"))
        return CardPreparer(codex, codex.attempts)
    if choice == "gemini":
        gemini = GeminiCli(os.environ.get("PHRASE_GYM_AGY", "agy"))
        return Recognizer(gemini, gemini.attempts)
    claude = ClaudeCli(os.environ.get("PHRASE_GYM_CLAUDE", "claude"))
    return CardPreparer(claude, claude.attempts)


async def prepare(work: dict[str, object], preparer: Preparer, speech: PiperSpeech) -> dict:
    claimed = CoachWork.model_validate(work)
    package_id = claimed.package_id
    try:
        speech.check_voice(claimed.learning_language)
    except MissingVoiceError as error:
        # A setup gap, not a language decision: the same text is prepared once a voice exists.
        raise PreparationError(message("no_voice", claimed.native_language)) from error
    card = await preparer.prepare(claimed)
    audio = await speech.synthesize(
        card.phrase.text if isinstance(card, PreparedPhrase) else card.text,
        claimed.learning_language,
    )
    result: dict[str, object] = {
        "result_id": package_id,
        "reference": base64.b64encode(audio).decode(),
    }
    if isinstance(card, BasicCard):
        result["basic"] = card.model_dump(mode="json", exclude_none=True)
        return result
    result["phrase"] = card.phrase.model_dump(mode="json", exclude_none=True)
    if card.correction:
        result["correction"] = card.correction.model_dump(mode="json")
        result["target"] = card.target
    return result


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


def process_once(api: CoachApi, preparer: Preparer, speech: PiperSpeech) -> bool:
    work = api.post("coach/next")
    if work is None:
        return False
    package_id = work["package_id"]
    details = ""
    retry_helps = True
    try:
        api.post(
            f"{package_id}/result",
            asyncio.run(prepare(work, preparer, speech)),
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
