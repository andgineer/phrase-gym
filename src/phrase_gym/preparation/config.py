"""Operator-owned provider configuration, separate from the served client."""

from phrase_gym.contracts.base import Contract
from phrase_gym.preparation.providers import ProviderSettings
from phrase_gym.preparation.speech import SpeechSettings


class PreparationSettings(Contract):
    provider: ProviderSettings
    speech: SpeechSettings

    def identity(self) -> dict:
        return self.model_dump(mode="json")
