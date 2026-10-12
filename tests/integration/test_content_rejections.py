import copy
from uuid import uuid4

import pytest

from tests.integration.helpers import phrases, publish, upload


@pytest.mark.parametrize(
    "mutation,message",
    [
        ("slot", "Frame slots"),
        ("required", "Required chunks"),
        ("duplicate_required", "Required chunks"),
        ("duplicate_cue", "Cue IDs"),
        ("duplicate_target", "Target source IDs"),
        ("duplicate_lesson", "Lesson/dialogue source IDs"),
        ("duplicate_dialogue", "Lesson/dialogue source IDs"),
        ("dialogue_target", "Dialogue target must exist"),
    ],
)
def test_broken_package_references_preserve_published_content(client, package, mutation, message):
    publish(client, package)
    previous = phrases(client)
    target = package["targets"][0]
    if mutation == "slot":
        target["slots"] = {"absent": ["replacement"]}
    elif mutation == "required":
        target["required_chunks"].append("absent")
    elif mutation == "duplicate_required":
        target["required_chunks"].append(target["required_chunks"][0])
    elif mutation == "duplicate_cue":
        target["cues"].append(copy.deepcopy(target["cues"][0]))
    elif mutation.startswith("duplicate_"):
        collection = {
            "duplicate_target": "targets",
            "duplicate_lesson": "lessons",
            "duplicate_dialogue": "dialogues",
        }[mutation]
        package[collection].append(copy.deepcopy(package[collection][0]))
    else:
        package["dialogues"][0]["turns"][1]["target_source_id"] = "absent"
    rejected = client.post("/api/v1/imports", json=package)
    assert rejected.status_code == 422
    assert message in rejected.text
    assert phrases(client) == previous


@pytest.mark.parametrize("kind", ["lesson", "dialogue"])
def test_language_change_requires_new_group_identity(client, package, kind):
    publish(client, package)
    previous = phrases(client)
    package["language"] = "sr"
    for target in package["targets"]:
        target["source_id"] = "new-" + target["source_id"]
    for lesson in package["lessons"]:
        lesson["target_source_ids"] = ["new-" + source for source in lesson["target_source_ids"]]
        if kind != "lesson":
            lesson["source_id"] = "new-" + lesson["source_id"]
    for dialogue in package["dialogues"]:
        if kind != "dialogue":
            dialogue["source_id"] = "new-" + dialogue["source_id"]
        for turn in dialogue["turns"]:
            if turn.get("target_source_id"):
                turn["target_source_id"] = "new-" + turn["target_source_id"]
    rejected = client.post("/api/v1/imports", json=package).json()
    assert rejected["status"] == "rejected"
    assert any(
        f"{kind.title()} " in error and "language changes" in error for error in rejected["errors"]
    )
    assert phrases(client) == previous
    assert phrases(client, lang="sr") == []


@pytest.mark.parametrize("missing", ["unknown", "lost"])
def test_missing_bound_media_cannot_publish(client, package, config, missing):
    publish(client, package)
    previous = phrases(client)
    if missing == "unknown":
        media_id = str(uuid4())
    else:
        asset = upload(client).json()
        media_id = asset["id"]
        (config.data_dir / "media" / asset["sha256"]).unlink()
    package["targets"][0]["reference_audio_id"] = media_id
    rejected = client.post("/api/v1/imports", json=package).json()
    assert rejected["status"] == "rejected"
    assert "missing reference media" in rejected["errors"][0]
    assert phrases(client) == previous
