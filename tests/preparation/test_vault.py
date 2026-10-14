from pathlib import Path

import pytest

from phrase_gym.contracts.vault import VaultManifest
from phrase_gym.errors import DomainError
from phrase_gym.integrations.legacy_notes import parse_note
from phrase_gym.integrations.vault import export_vault, confined, extract_block


def manifest(note="lesson.md"):
    return VaultManifest.model_validate(
        {
            "namespace": "vault",
            "language": "en",
            "native_language": "ru",
            "selections": [
                {
                    "source_id": "first",
                    "note": note,
                    "block_id": "a",
                    "meaning_key": "first",
                    "recall_expression": "Right.",
                    "purpose": "Agreement",
                },
                {
                    "source_id": "second",
                    "note": note,
                    "block_id": "b",
                    "meaning_key": "second",
                    "recall_expression": "Right.",
                    "purpose": "Direction",
                },
            ],
        }
    )


def test_read_only_stable_blocks_reorder_rename_and_distinct_meanings(tmp_path):
    vault = tmp_path / "vault"
    vault.mkdir()
    note = vault / "lesson.md"
    note.write_text("- Right. ^a\n- Right. ^b\n")
    before = note.read_bytes()
    exported, report = export_vault(vault, manifest())
    assert note.read_bytes() == before
    assert report["duplicate_surface_forms"] == ["Right."]
    note.rename(vault / "renamed.md")
    (vault / "renamed.md").write_text("- Right. ^b\n- Right. ^a\n")
    again, _ = export_vault(vault, manifest("renamed.md"))
    assert [(p.source_id, p.meaning_key) for p in exported.phrases] == [
        (p.source_id, p.meaning_key) for p in again.phrases
    ]


def test_missing_and_ambiguous_blocks_and_root_escape(tmp_path):
    with pytest.raises(DomainError, match="ambiguous"):
        extract_block("Hello ^a\nGoodbye ^a", "a")
    with pytest.raises(DomainError, match="ambiguous"):
        extract_block("No marker", "a")
    with pytest.raises(DomainError, match="within"):
        confined(tmp_path, Path("../outside"))


def test_legacy_format_phrases_context_dialogue_exact_target_links(tmp_path):
    note = """---
id: sample
курс: public-fixture
---
## фразы
### topic
1. Good to see you!
   ![[sample-01.mp3]]
   *Greet a friend.*
2. Thanks for coming.
   ![[sample-02.mp3]]
## диалоги
### Greeting
- A: **Good to see you!** ![[sample-01.mp3]]
- B: Thanks! ![[sample-d1-2-tts.mp3]] <!-- voice: en-GB-SoniaNeural -->
## notes
1. Ignore this unrelated section.
"""
    parsed = parse_note(note)
    assert len(parsed.phrases) == 2 and parsed.phrases[0].context == "Greet a friend."
    (tmp_path / "lesson.md").write_text(note)
    mapping = VaultManifest.model_validate(
        {
            "namespace": "vault",
            "language": "en",
            "native_language": "ru",
            "include_dialogues": True,
            "selections": [
                {
                    "source_id": "stable",
                    "note": "lesson.md",
                    "source_audio_name": "sample-01.mp3",
                    "meaning_key": "greeting",
                    "recall_expression": "Good to see you!",
                    "purpose": "Greet",
                }
            ],
        }
    )
    exported, _ = export_vault(tmp_path, mapping)
    assert exported.phrases[0].context == "Greet a friend."
    assert exported.dialogues[0].turns[0].target_source_id == "stable"
    assert exported.dialogues[0].turns[1].target_source_id is None
