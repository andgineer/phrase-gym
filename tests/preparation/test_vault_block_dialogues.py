import pytest

from phrase_gym.contracts.content import DialogueInput, DialogueTurn
from phrase_gym.contracts.vault import VaultManifest
from phrase_gym.integrations.vault import export_vault
from tests.preparation.test_vault_dialogue_assets import course_mapping


def test_block_and_audio_selectors_preserve_the_same_complete_exchange(tmp_path, source):
    manifest = course_mapping(tmp_path, source)
    manifest.assets = {}
    by_audio, audio_report = export_vault(tmp_path, manifest)
    manifest.selections[0].block_id = "greeting"
    manifest.selections[0].source_audio_name = None
    by_block, block_report = export_vault(tmp_path, manifest)
    assert not audio_report["errors"] and not block_report["errors"]
    assert by_block == by_audio
    turns = by_block.dialogues[0].turns
    assert [(t.speaker, t.text, t.target_source_id) for t in turns] == [
        ("A", "How are you?", None),
        ("B", source.phrases[0].text, "greeting"),
    ]


def test_mixed_selectors_parse_note_once_and_refuse_ambiguous_meaning_bindings(tmp_path, source):
    manifest = course_mapping(tmp_path, source)
    manifest.assets = {}
    extra = manifest.selections[0].model_copy(deep=True)
    extra.source_id = "second"
    extra.meaning_key = "second"
    extra.block_id = "second"
    extra.source_audio_name = None
    manifest.selections.append(extra)
    note = tmp_path / "lesson.md"
    note.write_text(
        note.read_text().replace(
            "## диалоги", "2. Goodbye. ^second\n   ![[goodbye.mp3]]\n## диалоги"
        )
    )
    package, report = export_vault(tmp_path, manifest)
    assert not report["errors"] and len(package.dialogues) == 1
    extra.block_id = "greeting"
    for selections in [manifest.selections, list(reversed(manifest.selections))]:
        manifest.selections = selections
        rejected, report = export_vault(tmp_path, manifest)
        assert (
            rejected is None
            and "Ambiguous dialogue target mapping: greeting, second"
            in report["errors"][0]["error"]
        )


@pytest.mark.parametrize("include_dialogues", [False, True])
def test_ordinary_block_notes_do_not_require_legacy_dialogue_format(tmp_path, include_dialogues):
    from_mapping = {
        "namespace": "blocks",
        "language": "en",
        "native_language": "ru",
        "include_dialogues": include_dialogues,
        "selections": [
            {
                "source_id": "plain",
                "note": "plain.md",
                "block_id": "plain",
                "meaning_key": "plain",
                "recall_expression": "Right.",
                "purpose": "Agree",
            }
        ],
    }
    # A plain block note needs no legacy front matter or course headings.
    (tmp_path / "plain.md").write_text("- Right. ^plain\n")
    manifest = VaultManifest.model_validate(from_mapping)
    package, report = export_vault(tmp_path, manifest)
    assert not report["errors"] and package.dialogues == []


def test_disabled_automatic_dialogues_preserve_explicit_bindings_without_parsing(tmp_path, source):
    manifest = course_mapping(tmp_path, source)
    manifest.assets = {}
    manifest.include_dialogues = False
    manifest.selections[0].block_id = "greeting"
    manifest.selections[0].source_audio_name = None
    note = tmp_path / "lesson.md"
    note.write_text(note.read_text().replace("id: public-course", "no-legacy-id: intentional"))
    manifest.dialogues = [
        DialogueInput(
            source_id="explicit",
            title="Reviewed complete exchange",
            turns=[
                DialogueTurn(speaker="A", text="How are you?"),
                DialogueTurn(speaker="B", text=source.phrases[0].text, target_source_id="greeting"),
            ],
        )
    ]
    package, report = export_vault(tmp_path, manifest)
    assert not report["errors"] and package.dialogues == manifest.dialogues
