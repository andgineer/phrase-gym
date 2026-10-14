"""Software-only generated-draft lifecycle against the real server and built client."""

import json

from phrase_gym.contracts.preparation import HumanReview, PreparationDraft, SourcePackage
from phrase_gym.preparation.jobs import digest
from phrase_gym.preparation.publication import publish_reviewed
from phrase_gym.preparation.worker import PreparationWorker
from tests.preparation.conftest import FakeProvider, FakeSpeech


def register_preparation_fixture(app, root):
    @app.post("/_test/prepared-greeting", include_in_schema=False)
    async def prepared_greeting():
        packet = json.loads((root / "tests/fixtures/preparation-rotation.json").read_text())
        jobs = app.state.preparation
        source = SourcePackage.model_validate(packet["source"])
        draft = PreparationDraft.model_validate(packet["draft"])
        queued = jobs.enqueue(source, {})
        worker = PreparationWorker(jobs, FakeProvider(draft), FakeSpeech(app.state.media))
        await worker.once()
        job = jobs.get(queued.id)
        assert job.status == "awaiting_review"
        review = HumanReview(
            input_hash=job.input_hash,
            draft_hash=digest(json.dumps(job.result)),
            reviewer="Software fixture only: synthetic tone; no actual listening claim",
            accepted_cues={"greeting": ["hide-a", "hide-c"]},
            help_reviewed=True,
            meaning_search_equivalents_reviewed=True,
            audio_reviewed=True,
            notes="Test only",
        )
        publish_reviewed(jobs, app.state.imports, job.id, review)
        return {"ready": True}
