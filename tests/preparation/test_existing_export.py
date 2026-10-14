from pathlib import Path

from click.testing import CliRunner

from phrase_gym.contracts.content import DialogueInput, DialogueTurn
from phrase_gym.contracts.preparation import SourcePackage
from phrase_gym.main import phrase_gym
from phrase_gym.preparation.publication import publish_reviewed
from tests.preparation.test_publication import completed


def test_existing_export_preserves_latest_complete_dialogue_and_audio(
    prepared_runtime, source, draft, tmp_path
):
    jobs, _, imports, _ = prepared_runtime
    source.dialogues = [
        DialogueInput(
            source_id="exchange",
            title="Greeting exchange",
            turns=[
                DialogueTurn(speaker="A", text="How are you?"),
                DialogueTurn(speaker="B", text=source.phrases[0].text, target_source_id="greeting"),
            ],
        )
    ]
    job, review = completed(prepared_runtime, source, draft)
    publish_reviewed(jobs, imports, job.id, review)
    source.source_revision = "2"
    source.dialogues[0].title = "Updated exchange title"
    changed, approval = completed(prepared_runtime, source, draft)
    publish_reviewed(jobs, imports, changed.id, approval)
    output = tmp_path / "source.json"
    data_dir = Path(jobs.db.engine.url.database).parent
    response = CliRunner().invoke(
        phrase_gym,
        ["--data-dir", str(data_dir), "export-existing", source.namespace, "ru", str(output)],
    )
    assert response.exit_code == 0, response.output
    exported = SourcePackage.model_validate_json(output.read_text())
    assert len(exported.dialogues) == 1
    assert exported.dialogues[0].title == "Updated exchange title"
    assert exported.dialogues[0].turns[1].target_source_id == "greeting"
    assert all(turn.audio_id for turn in exported.dialogues[0].turns)
    assert exported.phrases[0].reference_audio_id
