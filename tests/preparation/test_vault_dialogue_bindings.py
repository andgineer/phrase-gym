import asyncio
import json

import pytest
from sqlalchemy import select

from phrase_gym.contracts.content import (
    Chunk,
    DialogueInput,
    DialogueTurn,
    ManualPackage,
    TargetInput,
)
from phrase_gym.contracts.vault import VaultManifest
from phrase_gym.integrations.vault import export_vault
from phrase_gym.tables import DialogueRow, ScheduleRow, Target
from tests.integration.helpers import NOW
from tests.preparation.conftest import FakeSpeech

NOTE = """---
id: sample
---
## фразы
1. Right.
   ![[right.mp3]]
## диалоги
### Two meanings
- A: **Right.** ![[right.mp3]]
- B: Where should I turn? ![[partner.mp3]]
"""


def mapping():
    return VaultManifest.model_validate(
        {
            "namespace": "vault",
            "language": "en",
            "native_language": "ru",
            "include_dialogues": True,
            "selections": [
                {
                    "source_id": meaning,
                    "note": "lesson.md",
                    "source_audio_name": "right.mp3",
                    "meaning_key": meaning,
                    "recall_expression": "Right.",
                    "purpose": purpose,
                }
                for meaning, purpose in [("agreement", "Agree"), ("direction", "Name a direction")]
            ],
        },
    )


def test_ambiguous_automatic_binding_is_rejected_independently_of_manifest_order(tmp_path):
    note = tmp_path / "lesson.md"
    note.write_text(NOTE, encoding="utf-8")
    manifest = mapping()
    results = []
    for selections in [manifest.selections, list(reversed(manifest.selections))]:
        manifest.selections = selections
        package, report = export_vault(tmp_path, manifest)
        assert package is None
        assert report["duplicate_surface_forms"] == ["Right."]
        assert report["errors"][0]["source_id"] == "dialogues"
        assert (
            "Ambiguous dialogue target mapping: agreement, direction"
            in report["errors"][0]["error"]
        )
        results.append(report["errors"])
    assert results[0] == results[1] and note.read_text(encoding="utf-8") == NOTE


@pytest.mark.parametrize("match", [False, True])
def test_zero_matches_remain_partner_context_and_unique_matches_link_correctly(tmp_path, match):
    (tmp_path / "lesson.md").write_text(NOTE, encoding="utf-8")
    manifest = mapping()
    manifest.selections = manifest.selections[:1]
    if not match:
        manifest.selections[0].block_id = "block"
        (tmp_path / "lesson.md").write_text(NOTE + "\nRight. ^block\n", encoding="utf-8")
        # Parse the note through a second, differently attached selection.
        extra = manifest.selections[0].model_copy(deep=True)
        extra.source_id = "extra"
        extra.meaning_key = "extra"
        extra.block_id = None
        extra.source_audio_name = "elsewhere.mp3"
        manifest.selections[0].source_audio_name = None
        manifest.selections.append(extra)
        (tmp_path / "lesson.md").write_text(
            NOTE.replace("## диалоги", "2. Elsewhere.\n   ![[elsewhere.mp3]]\n## диалоги")
            + "\nRight. ^block\n",
            encoding="utf-8",
        )
    package, report = export_vault(tmp_path, manifest)
    assert not report["errors"]
    first, second = package.dialogues[0].turns
    assert first.target_source_id == ("agreement" if match else None)
    assert second.target_source_id is None


def test_explicit_complete_dialogue_preserves_distinct_meaning_targets_and_schedules(
    tmp_path, prepared_runtime
):
    jobs, media, imports, _ = prepared_runtime
    reference = asyncio.run(FakeSpeech(media).prepare("Right.", "en", "vault:synthetic"))
    (tmp_path / "lesson.md").write_text(NOTE, encoding="utf-8")
    manifest = mapping()
    manifest.include_dialogues = False
    manifest.dialogues = [
        DialogueInput(
            source_id="explicit",
            title="Reviewed meaning bindings",
            turns=[
                DialogueTurn(
                    speaker="A", text="Right.", target_source_id="agreement", audio_id=reference
                ),
                DialogueTurn(
                    speaker="B", text="Right.", target_source_id="direction", audio_id=reference
                ),
            ],
        ),
    ]
    exported, report = export_vault(tmp_path, manifest)
    assert not report["errors"] and len(exported.dialogues) == 1
    assert [turn.target_source_id for turn in exported.dialogues[0].turns] == [
        "agreement",
        "direction",
    ]
    targets = [
        TargetInput(
            source_id=phrase.source_id,
            kind=phrase.kind,
            meaning_key=phrase.meaning_key,
            recall_expression=phrase.recall_expression,
            text=phrase.text,
            purpose=phrase.purpose,
            chunks=[Chunk(id="whole", text=phrase.text)],
            required_chunks=["whole"],
        )
        for phrase in exported.phrases
    ]
    # No cues: identities/bindings can be published without certifying generated language content.
    package = ManualPackage(
        namespace=exported.namespace,
        language=exported.language,
        source_revision=exported.source_revision,
        targets=targets,
        dialogues=exported.dialogues,
    )
    staged = imports.stage(package)
    assert not staged.errors
    imports.publish(staged.id)
    with jobs.db.write() as transaction:
        stored = transaction.scalars(select(Target)).all()
        assert len(stored) == 2 and len({target.id for target in stored}) == 2
        for position, target in enumerate(stored):
            transaction.add(
                ScheduleRow(
                    target_id=target.id,
                    version=position + 1,
                    due_at=NOW.isoformat(),
                    memory_state=None,
                    policy_version=None,
                )
            )
        transaction.flush()
        schedules = transaction.scalars(select(ScheduleRow)).all()
        assert len(schedules) == 2 and {row.version for row in schedules} == {1, 2}
        binding = json.loads(transaction.scalar(select(DialogueRow)).target_ids)["ids"]
        assert binding["agreement"] != binding["direction"]
