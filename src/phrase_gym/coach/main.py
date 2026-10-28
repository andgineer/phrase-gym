"""Foreground Coach that prepares only work obtained from the Phrase Gym API."""

import asyncio
import base64
import json
import time
import urllib.error
import urllib.request
from typing import Any
from urllib.parse import urlsplit

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


async def prepare(work: dict[str, Any], speech: PiperSpeech) -> dict[str, str]:
    return {
        card["item_id"]: base64.b64encode(await speech.synthesize(card["answer"])).decode()
        for card in work["package"]["cards"]
    }


def process_once(api: CoachApi, speech: PiperSpeech) -> bool:
    work = api.post("coach/next")
    if work is None:
        return False
    package_id = work["package_id"]
    try:
        references = asyncio.run(prepare(work, speech))
        api.post(f"{package_id}/result", {"result_id": package_id, "references": references})
    except (OSError, ValueError, RuntimeError, urllib.error.HTTPError) as error:
        api.post(f"{package_id}/failure", {"error": str(error)[:1000]})
    return True


def run_coach(url: str, *, once: bool = False) -> None:
    api = CoachApi(url)
    speech = configured_speech()
    while True:
        try:
            found = process_once(api, speech)
        except (OSError, urllib.error.HTTPError) as error:
            print(f"Coach cannot reach Phrase Gym: {error}", flush=True)
            found = False
        if once:
            return
        if not found:
            time.sleep(2)
