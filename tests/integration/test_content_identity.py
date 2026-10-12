import hashlib

from phrase_gym.storage import Database, TargetRevision
from tests.integration.helpers import phrases, publish


def test_object_key_order_preserves_existing_revisions(client, package, config):
    publish(client, package)
    original = phrases(client)
    database = Database(config.data_dir)
    with database.write() as session:
        for item in original:
            row = session.get(TargetRevision, (item["id"], item["revision"]))
            row.fingerprint = hashlib.sha256(row.payload.encode()).hexdigest()
    database.close()
    for target in package["targets"]:
        target["explanations"] = dict(reversed(list(target["explanations"].items())))
        for explanation in target["explanations"].values():
            reordered = dict(reversed(list(explanation.items())))
            explanation.clear()
            explanation.update(reordered)
    report = publish(client, package)
    assert all(change["action"] == "unchanged" for change in report["changes"])
    assert phrases(client) == original


def test_array_order_remains_part_of_revision_identity(client, package):
    package["targets"][0]["cues"].append(
        {**package["targets"][0]["cues"][0], "id": "another-reviewed-cue"},
    )
    publish(client, package)
    previous = phrases(client, q="greetings")[0]
    package["targets"][0]["cues"].reverse()
    assert len(package["targets"][0]["cues"]) > 1
    report = publish(client, package)
    assert report["changes"][0]["action"] == "update"
    latest = phrases(client, q="greetings")[0]
    assert latest["id"] == previous["id"]
    assert latest["revision"] == previous["revision"] + 1
    assert latest["content"]["cues"] == list(reversed(previous["content"]["cues"]))
