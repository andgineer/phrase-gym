"""Explicit llmbroker pool/direct configuration with no implicit paid escalation."""

import asyncio
import time
from dataclasses import asdict
from pathlib import Path

from llmbroker import AsyncBroker, LLMBrokerError, LLMRequestError, Secrets
from pydantic import Field

from phrase_gym.contracts.base import Contract
from phrase_gym.errors import DomainError


class ProviderSettings(Contract):
    keys_file: Path | None = None
    direct_model: str | None = None
    allow_paid: bool = False
    wait_seconds: float = Field(default=60, ge=1, le=120)


class BrokerProvider:
    def __init__(self, settings: ProviderSettings, home: Path):
        if settings.direct_model and not settings.allow_paid:
            raise DomainError("Direct paid model requires deliberate allow_paid=true", 422)
        self.settings = settings
        self.broker = AsyncBroker(
            home=home,
            direct=[settings.direct_model] if settings.direct_model else [],
            secrets=Secrets(env_file=settings.keys_file) if settings.keys_file else Secrets(),
        )

    async def availability(self) -> dict:
        snapshot = await self.broker.snapshot()
        return {
            "providers_usable": snapshot.providers_usable,
            "models": [
                {
                    "name": name,
                    "has_key": model.has_key,
                    "disabled": model.disabled,
                    "cooldown_until": model.cooldown_until.isoformat()
                    if model.cooldown_until
                    else None,
                }
                for name, model in snapshot.items()
            ],
        }

    async def ask(self, prompt: str, operation: str) -> dict:
        started = time.monotonic()
        try:
            if self.settings.direct_model:
                client = await self.broker.direct(self.settings.direct_model)
                result = await asyncio.wait_for(
                    client.ask(prompt, timeout=self.settings.wait_seconds),
                    timeout=self.settings.wait_seconds,
                )
                model = self.settings.direct_model
            else:
                result = await asyncio.wait_for(
                    self.broker.ask(
                        prompt,
                        operation=operation,
                        wait=self.settings.wait_seconds,
                        fastest_of=1,
                    ),
                    timeout=self.settings.wait_seconds,
                )
                model = result.llm_name
        except (LLMBrokerError, LLMRequestError, TimeoutError) as error:
            raise DomainError(
                "provider_unavailable: bounded request did not complete",
                503,
            ) from error
        return {
            "text": result.text,
            "provider": "llmbroker",
            "model": model,
            "latency_seconds": round(time.monotonic() - started, 3),
            "usage": asdict(result.usage) if result.usage else None,
        }

    async def close(self) -> None:
        await self.broker.aclose()
