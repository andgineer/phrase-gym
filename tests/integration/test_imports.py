from phrase_gym.content import publication
from concurrent.futures import ThreadPoolExecutor
import copy
import pytest
from sqlalchemy import func, select
from phrase_gym.storage import Database
from phrase_gym.tables import TargetRevision
from tests.integration.helpers import publish, phrases


def test_dry_run_stage_explicit_publish_and_retry(client, package):
    validation = client.post("/api/v1/imports/validate", json=package)
    assert validation.json()["status"] == "validated"
    assert phrases(client) == []
    staged = client.post("/api/v1/imports", json=package).json()
    assert phrases(client) == []
    url = f"/api/v1/imports/{staged['id']}/publish"
    assert client.post(url, json={"reviewed": False}).status_code == 422
    first = client.post(url, json={"reviewed": True}).json()
    assert all(change["target_id"] for change in first["changes"])
    assert client.post(url, json={"reviewed": True}).json() == first
    assert len(phrases(client)) == 3


def test_repeat_import_reorder_rename_stable_identity(client, package, config):
    publish(client, package)
    original = {p["content"]["source_id"]: (p["id"], p["revision"]) for p in phrases(client)}
    lessons = client.get("/api/v1/lessons?language=en").json()["items"]
    package["targets"].reverse()
    package["lessons"][0]["title"] = "Rapport, renamed"
    package["lessons"][0]["target_source_ids"].reverse()
    package["source_revision"] = "2"
    report = publish(client, package)
    assert all(change["action"] == "unchanged" for change in report["changes"])
    assert {
        p["content"]["source_id"]: (p["id"], p["revision"]) for p in phrases(client)
    } == original
    renamed = client.get("/api/v1/lessons?language=en&q=renamed").json()["items"][0]
    assert renamed["id"] in {l["id"] for l in lessons}
    database = Database(config.data_dir)
    with database.read() as session:
        assert session.scalar(select(func.count()).select_from(TargetRevision)) == 3
    database.close()


def test_editorial_revision_retains_old_content_and_identity(client, package):
    publish(client, package)
    previous = phrases(client, q="greetings")[0]
    package["targets"][0]["explanations"]["ru"]["usage"] = "Новое пояснение."
    publish(client, package)
    latest = phrases(client, q="greetings")[0]
    assert latest["id"] == previous["id"] and latest["revision"] == 2
    old = client.get(f"/api/v1/phrases/{latest['id']}?language=en&revision=1").json()
    assert old["content"] == previous["content"]


@pytest.mark.parametrize(
    "field,value",
    [
        ("meaning_key", "different-sense"),
        ("kind", "word_sense"),
        ("recall_expression", "another expression"),
    ],
)
def test_changed_objective_requires_new_identity(client, package, field, value):
    publish(client, package)
    previous = phrases(client)
    package["targets"][0][field] = value
    report = client.post("/api/v1/imports", json=package).json()
    assert report["status"] == "rejected"
    assert "new source ID" in report["errors"][0]
    assert phrases(client) == previous


@pytest.mark.parametrize(
    "mutation",
    ["unknown_chunk", "joined_text", "unreviewed", "unhidden", "lesson_member", "dialogue_text"],
)
def test_invalid_content_never_publishes(client, package, mutation):
    if mutation == "unknown_chunk":
        package["targets"][0]["cues"][0]["visible_chunks"].append("absent")
    if mutation == "joined_text":
        package["targets"][0]["chunks"][0]["text"] = "Wrong "
    if mutation == "unreviewed":
        package["targets"][0]["cues"][0]["reviewed"] = False
    if mutation == "unhidden":
        package["targets"][0]["cues"][0]["visible_chunks"] = ["opening", "subject", "ending"]
    if mutation == "lesson_member":
        package["lessons"][0]["target_source_ids"].append("absent")
    if mutation == "dialogue_text":
        package["dialogues"][0]["turns"][1]["text"] = "Different phrase"
    assert client.post("/api/v1/imports", json=package).status_code == 422
    assert phrases(client) == []


def test_stale_target_or_lesson_publication_rejected(client, package):
    publish(client, package)
    older = copy.deepcopy(package)
    older["lessons"][0]["title"] = "Old pending edit"
    staged = client.post("/api/v1/imports", json=older).json()
    package["lessons"][0]["title"] = "New lesson edit"
    publish(client, package)
    response = client.post(f"/api/v1/imports/{staged['id']}/publish", json={"reviewed": True})
    assert response.status_code == 409
    assert client.get("/api/v1/lessons?language=en&q=New").json()["total"] == 1


def test_publication_failure_rolls_back_whole_package(client, package, monkeypatch):
    staged = client.post("/api/v1/imports", json=package).json()

    def failed_dialogues(*_args):
        raise RuntimeError("Simulated publication crash")

    monkeypatch.setattr(publication, "publish_dialogues", failed_dialogues)
    with pytest.raises(RuntimeError, match="Simulated publication crash"):
        client.post(f"/api/v1/imports/{staged['id']}/publish", json={"reviewed": True})
    assert phrases(client) == []
    assert client.get("/api/v1/lessons?language=en").json()["items"] == []
    assert client.get(f"/api/v1/imports/{staged['id']}").json()["status"] == "staged"


def test_parallel_publication_consumes_source_identity_once(client, package):
    staged = client.post("/api/v1/imports", json=package).json()
    url = f"/api/v1/imports/{staged['id']}/publish"
    with ThreadPoolExecutor(max_workers=2) as executor:
        responses = list(
            executor.map(lambda _: client.post(url, json={"reviewed": True}), range(2))
        )
    assert [r.status_code for r in responses] == [200, 200]
    assert responses[0].json() == responses[1].json()
    assert len(phrases(client)) == 3
