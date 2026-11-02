import io
import urllib.error
from email.message import Message
from typing import Any

from fastapi.testclient import TestClient


class ClientCoachApi:
    """Routes Coach's API calls to the in-process app with urllib's error behavior."""

    def __init__(self, client: TestClient):
        self.client = client

    def post(self, path: str, payload: dict[str, object] | None = None) -> Any:
        response = self.client.post(f"/api/imports/{path}", json=payload or {})
        if response.status_code >= 400:
            raise urllib.error.HTTPError(
                path, response.status_code, "Conflict", Message(), io.BytesIO(response.content)
            )
        return response.json()
