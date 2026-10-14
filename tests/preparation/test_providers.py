import asyncio

import pytest
from llmbroker import NoLLMAvailableError

from phrase_gym.errors import DomainError
from phrase_gym.preparation.providers import BrokerProvider, ProviderSettings


def test_pool_timeout_maps_to_recoverable_error_without_paid_escalation(tmp_path, monkeypatch):
    calls = []

    class FailedBroker:
        def __init__(self, **configuration):
            assert configuration["direct"] == []

        async def ask(self, *args, **kwargs):
            calls.append(kwargs)
            raise NoLLMAvailableError("Test unavailable", reason="timeout")

        async def aclose(self):
            pass

    monkeypatch.setattr("phrase_gym.preparation.providers.AsyncBroker", FailedBroker)
    provider = BrokerProvider(ProviderSettings(), tmp_path)
    with pytest.raises(DomainError, match="bounded request"):
        asyncio.run(provider.ask("public fixture", "test"))
    assert calls[0]["fastest_of"] == 1
