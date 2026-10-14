import asyncio
from pathlib import Path
from uuid import UUID

import pytest
from click.testing import CliRunner

from phrase_gym.contracts.vault import VaultManifest
from phrase_gym.integrations.vault import export_vault
from phrase_gym.main import phrase_gym
from phrase_gym.preparation.worker import PreparationWorker
from tests.integration.helpers import wav_bytes
from tests.preparation.conftest import FakeProvider, FakeSpeech


def course_mapping(tmp_path, source):
    note = """---
id: public-course
---
## фразы
1. Not too bad, thanks. And yours? ^greeting
   ![[reference.mp3]]
## диалоги
### Complete exchange
- A: How are you? ![[partner.wav]]
- B: Not too bad, thanks. And yours? ![[reference.mp3]]
"""
    (tmp_path / "lesson.md").write_text(note)
    return VaultManifest.model_validate(
        {
            "namespace": source.namespace,
            "language": "en",
            "native_language": "ru",
            "include_dialogues": True,
            "selections": [
                {
                    "source_id": "greeting",
                    "note": "lesson.md",
                    "source_audio_name": "reference.mp3",
                    "meaning_key": "greeting",
                    "recall_expression": source.phrases[0].text,
                    "purpose": source.phrases[0].purpose,
                }
            ],
            "assets": {
                "partner.wav": {
                    "path": "partner.wav",
                    "mime_type": "audio/wav",
                    "metadata": {
                        "language": "en",
                        "text": "How are you?",
                        "role": "partner",
                        "origin": "course",
                        "source": "public synthetic course",
                        "performer": "fixture actor",
                        "locale": "en-US",
                        "preparation_version": "original-1",
                    },
                }
            },
        },
    )


@pytest.mark.parametrize("failure", ["missing", "escape", "symlink", "corrupt"])
def test_dry_run_validates_every_referenced_dialogue_asset(tmp_path, source, failure):
    vault = tmp_path / "vault"
    vault.mkdir()
    manifest = course_mapping(vault, source)
    asset = manifest.assets["partner.wav"]
    if failure == "escape":
        (tmp_path / "outside.wav").write_bytes(wav_bytes())
        asset.path = Path("../outside.wav")
    elif failure == "symlink":
        outside = tmp_path / "outside.wav"
        outside.write_bytes(wav_bytes())
        (vault / "partner.wav").symlink_to(outside)
    elif failure == "corrupt":
        (vault / "partner.wav").write_bytes(b"not an audio container")
    before = (vault / "lesson.md").read_bytes()
    package, report = export_vault(vault, manifest)
    assert package is None and report["errors"][0]["source_id"] == "dialogues"
    expected = (
        "within"
        if failure in ("escape", "symlink")
        else "Invalid audio"
        if failure == "corrupt"
        else "No such file"
    )
    assert expected in report["errors"][0]["error"]
    assert (vault / "lesson.md").read_bytes() == before


def test_valid_unuploaded_partner_asset_is_reported_and_cannot_be_exported(tmp_path, source):
    vault = tmp_path / "vault"
    vault.mkdir()
    manifest = course_mapping(vault, source)
    (vault / "partner.wav").write_bytes(wav_bytes())
    package, report = export_vault(vault, manifest)
    assert not report["errors"] and package.dialogues[0].turns[0].audio_id is None
    assert report["assets_not_uploaded"] == ["dialogue_asset:partner.wav"]
    mapping = tmp_path / "manifest.json"
    mapping.write_text(manifest.model_dump_json())
    output = tmp_path / "export.json"
    response = CliRunner().invoke(
        phrase_gym,
        [
            "--data-dir",
            str(tmp_path / "data"),
            "export-vault",
            str(vault),
            str(mapping),
            "--output",
            str(output),
        ],
    )
    assert response.exit_code != 0 and "--upload-assets" in response.output
    assert "dialogue_asset:partner.wav" in response.output and not output.exists()


@pytest.mark.parametrize("origin", ["course", "actor"])
def test_upload_retains_original_partner_bytes_and_worker_does_not_replace_clip(
    tmp_path, prepared_runtime, source, draft, origin
):
    jobs, media, _, _ = prepared_runtime
    vault = tmp_path / "vault"
    vault.mkdir()
    manifest = course_mapping(vault, source)
    manifest.assets["partner.wav"].metadata.origin = origin
    original = wav_bytes()
    (vault / "partner.wav").write_bytes(original)
    package, report = export_vault(vault, manifest, media)
    assert report["assets_not_uploaded"] == [] and not report["errors"]
    partner = package.dialogues[0].turns[0].audio_id
    path, metadata = media.delivery(partner)
    assert path.read_bytes() == original
    assert metadata.metadata.origin == origin and metadata.metadata.performer == "fixture actor"
    provider, speech = FakeProvider(draft), FakeSpeech(media)
    job = jobs.enqueue(package, {})
    assert asyncio.run(PreparationWorker(jobs, provider, speech).once(job.id))
    result = jobs.get(job.id)
    assert result.status == "awaiting_review" and speech.calls == 1
    turns = result.result["package"]["dialogues"][0]["turns"]
    assert UUID(turns[0]["audio_id"]) == partner
    assert "dialogue:public-course.dialogue.0:0" not in result.result["audio"]
    assert (vault / "partner.wav").read_bytes() == original
    old_revision = package.source_revision
    (vault / "partner.wav").write_bytes(wav_bytes() + b"supplemental original bytes")
    revised, _ = export_vault(vault, manifest, media)
    assert revised.source_revision != old_revision
