from uuid import uuid4

import pytest

from phrase_gym.contracts.handoff import HandoffEnvelope
from phrase_gym.errors import DomainError
from phrase_gym.integrations.handoff import HandoffReceiver


def test_durable_receipt_retry_restart_and_changed_sense(prepared_runtime, source):
    jobs, _, _, _ = prepared_runtime
    envelope = HandoffEnvelope(
        export_id=uuid4(),
        sender="echo-words",
        selected_sense_id="greeting",
        selected_sense="Greeting",
        package=source,
    )
    receiver = HandoffReceiver(jobs, {})
    receipt = receiver.receive(envelope)
    assert receipt.durable and receipt.status == "received"
    assert HandoffReceiver(jobs, {}).receive(envelope) == receipt
    envelope.selected_sense = "Different selected meaning"
    with pytest.raises(DomainError, match="different selected"):
        receiver.receive(envelope)
    assert receiver.status(envelope.export_id) == receipt
