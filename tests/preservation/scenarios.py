"""Replay language-neutral HTTP preservation scenarios against the Python server."""

import base64
import copy
import json
import sqlite3
from datetime import datetime
from pathlib import Path

from fastapi.testclient import TestClient

from phrase_gym.api.app import create_app
from phrase_gym.config import Config
from tests.integration.helpers import ROOT

SCENARIOS = ROOT / "fixtures/preservation/scenarios"


class ScenarioClock:
    def __init__(self, now: str):
        self.now = datetime.fromisoformat(now)

    def __call__(self) -> datetime:
        return self.now


def scenario_names() -> list[str]:
    return sorted(path.stem for path in SCENARIOS.glob("*.json"))


def load(name: str) -> dict:
    return json.loads((SCENARIOS / f"{name}.json").read_text(encoding="utf-8"))


def pointer_parts(pointer: str) -> list[str]:
    if not pointer:
        return []
    assert pointer.startswith("/"), pointer
    return [part.replace("~1", "/").replace("~0", "~") for part in pointer[1:].split("/")]


def resolve(document, pointer: str):
    for part in pointer_parts(pointer):
        document = document[int(part)] if isinstance(document, list) else document[part]
    return document


def assign(document, pointer: str, value) -> None:
    *parents, last = pointer_parts(pointer)
    target = resolve(document, "".join(f"/{part}" for part in parents))
    if isinstance(target, list):
        target[int(last)] = value
    else:
        target[last] = value


def scenario_bytes(spec: dict) -> bytes:
    source = {key: value for key, value in spec.items() if key != "slice"}
    (kind, value), *rest = source.items()
    assert not rest, spec
    if kind == "file":
        data = (ROOT / value).read_bytes()
    elif kind == "fill":
        data = bytes([value[0]]) * value[1]
    elif kind == "text":
        data = value.encode()
    else:
        raise AssertionError(f"Unknown byte source {kind}")
    start, end = spec.get("slice", [0, len(data)])
    return data[start:end]


def expand(value):
    if isinstance(value, list):
        return [expand(item) for item in value]
    if not isinstance(value, dict):
        return value
    if "$base64" in value:
        return base64.b64encode(scenario_bytes(value["$base64"])).decode()
    if "$json_file" in value:
        document = json.loads((ROOT / value["$json_file"]).read_text(encoding="utf-8"))
        for pointer, replacement in value.get("set", {}).items():
            assign(document, pointer, expand(replacement))
        return document
    return {key: expand(item) for key, item in value.items()}


def substitute(value, captures: dict):
    if isinstance(value, list):
        return [substitute(item, captures) for item in value]
    if isinstance(value, dict):
        return {key: substitute(item, captures) for key, item in value.items()}
    if isinstance(value, str):
        if value.startswith("{{") and value.endswith("}}") and value.count("{{") == 1:
            return copy.deepcopy(captures[value[2:-2]])
        for name, captured in captures.items():
            value = value.replace(f"{{{{{name}}}}}", str(captured))
        assert "{{" not in value, f"Unresolved capture in {value}"
    return value


def check_json(step: str, body, checks: list[dict], responses: dict) -> None:
    for check in checks:
        where = f"{step} {check['at']}"
        if check.get("absent"):
            parent, _, key = check["at"].rpartition("/")
            assert key not in resolve(body, parent), where
            continue
        actual = resolve(body, check["at"])
        if "equals" in check:
            assert actual == check["equals"], f"{where}: {actual!r}"
        if "same_as" in check:
            source, _, pointer = check["same_as"].partition(":")
            assert actual == resolve(responses[source], pointer), f"{where}: {actual!r}"
        if "instant" in check:
            expected = datetime.fromisoformat(check["instant"])
            assert datetime.fromisoformat(actual) == expected, f"{where}: {actual!r}"
        if "length" in check:
            assert len(actual) == check["length"], f"{where}: {actual!r}"


def send(client: TestClient, request: dict):
    headers = dict(request.get("headers", {}))
    if "raw" in request:
        headers.setdefault("content-type", "application/json")
        return client.request(
            request["method"], request["path"], content=request["raw"], headers=headers
        )
    return client.request(
        request["method"], request["path"], json=request.get("json"), headers=headers
    )


def run(name: str, data_dir: Path) -> None:
    scenario = load(name)
    assert "python" in scenario["implementations"], name
    clock = ScenarioClock(scenario["clock"])
    with TestClient(create_app(Config(data_dir), clock=clock)) as client:
        if scenario.get("seed_sql"):
            connection = sqlite3.connect(data_dir / "phrase-gym.sqlite3")
            try:
                connection.executescript("\n".join(scenario["seed_sql"]))
            finally:
                connection.close()
        captures: dict = {}
        responses: dict = {}
        for step in scenario["steps"]:
            if "clock" in step:
                clock.now = datetime.fromisoformat(step["clock"])
            request = substitute(expand(step["request"]), captures)
            response = send(client, request)
            expect = substitute(expand(step["expect"]), captures)
            assert response.status_code == expect["status"], (step["id"], response.text)
            for header, value in expect.get("headers", {}).items():
                assert response.headers.get(header) == value, (step["id"], header)
            if "bytes" in expect:
                assert response.content == scenario_bytes(expect["bytes"]), step["id"]
                continue
            body = response.json() if response.content else None
            responses[step["id"]] = body
            check_json(step["id"], body, expect.get("json", []), responses)
            for capture, pointer in step.get("capture", {}).items():
                captures[capture] = resolve(body, pointer)
