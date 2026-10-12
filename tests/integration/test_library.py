import json
from tests.integration.helpers import ROOT, publish, phrases


def test_lookup_lesson_memberships_and_languages(client, package):
    publish(client, package)
    serbian = json.loads((ROOT / "fixtures/manual-sr.json").read_text(encoding="utf-8"))
    publish(client, serbian)
    lesson = client.get("/api/v1/lessons?language=en&q=rap").json()["items"][0]
    result = phrases(client, q="greetings")
    assert len(result) == 1 and "greetings" not in result[0]["content"]["text"]
    assert len(phrases(client, "sr", "greetings")) == 1
    assert len(phrases(client, "sr", "Drago")) == 1
    english_id = result[0]["id"]
    assert client.get(f"/api/v1/phrases/{english_id}?language=sr").status_code == 404
    lesson_url = f"/api/v1/lessons/{lesson['id']}/phrases"
    assert client.get(lesson_url + "?language=sr").status_code == 404
    members = client.get(lesson_url + "?language=en").json()
    assert members["total"] == 2
    assert members["items"][0]["content"]["source_id"] == "day"
    shared = phrases(client, q="coffee")[0]
    assert len(shared["lessons"]) == 2
    assert client.get("/api/v1/phrases/search?language=en&limit=1&offset=1").json()["total"] == 3
    assert phrases(client, q="nothing-in-this-library") == []


def test_dialogue_revisions_pin_target_content_without_extra_targets(client, package):
    publish(client, package)
    dialogues = client.get("/api/v1/dialogues?language=en").json()
    first = dialogues["items"][0]
    assert dialogues["total"] == 1
    target_id = first["target_ids"]["catch-up"]
    assert first["target_revisions"]["catch-up"] == 1
    assert target_id == phrases(client, q="coffee")[0]["id"]
    package["targets"][1]["explanations"]["en"]["usage"] = "An editorial revision."
    publish(client, package)
    latest = client.get(f"/api/v1/dialogues/{first['id']}?language=en").json()
    assert latest["revision"] == 2
    assert latest["target_revisions"]["catch-up"] == 2
    old = client.get(f"/api/v1/dialogues/{first['id']}?language=en&revision=1").json()
    assert old == first
    assert len(phrases(client)) == 3
    assert client.get(f"/api/v1/dialogues/{first['id']}?language=sr").status_code == 404
