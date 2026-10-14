"""Portable selected-sense envelope; audio travels through verified upload IDs."""

from typing import Literal
from uuid import UUID

from phrase_gym.contracts.base import Contract, Identifier, Text
from phrase_gym.contracts.preparation import SourcePackage


class HandoffEnvelope(Contract):
    schema_version: Literal[1] = 1
    export_id: UUID
    sender: Identifier
    selected_sense_id: Identifier
    selected_sense: Text
    package: SourcePackage


class HandoffReceipt(Contract):
    export_id: UUID
    preparation_id: UUID
    status: Literal["received", "preparing", "ready", "failed"]
    durable: Literal[True] = True
