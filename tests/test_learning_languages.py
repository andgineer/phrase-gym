from datetime import UTC, datetime
from typing import Any

import pytest
from fastapi.testclient import TestClient

from phrase_gym.coach.main import process_once
from phrase_gym.coach.speech import PiperSpeech
from phrase_gym.server.coach_contracts import BasicCard, CoachWork
from phrase_gym.server.coach_requests import request_id
from tests.coach_client import ClientCoachApi
from tests.phrase_packages import (
    Package,
    lesson,
    make_package,
    phrase,
    publish,
    references,
    silent_reference,
)
from tests.practice_client import Practice

# Monday 10:00 in Europe/Belgrade.
MONDAY = datetime(2026, 11, 2, 9, tzinfo=UTC)


@pytest.fixture
def practice(client: TestClient) -> Practice:
    return Practice(client, now=MONDAY)


def english(count: int) -> Package:
    return make_package(lessons=[lesson("unit-1", "Meetings", [phrase(i) for i in range(count)])])


def german(count: int, **changes: str) -> Package:
    cards = [
        phrase(i, item_id=f"alltag-{i}", text=f"Das ist der Tisch Nummer {i}.")
        for i in range(count)
    ]
    package = make_package(
        package_id="deutsch-a1-v1",
        source_id="deutsch-a1",
        language="de",
        lessons=[lesson("alltag", "Alltag", cards)],
    )
    return package | changes


def finish_daily(practice: Practice, language: str) -> list[dict[str, Any]]:
    finished = []
    state = practice.daily(language=language)
    while state["card"]:
        card = state["card"]
        finished.append(card)
        outcome = "not_yet" if card["kind"] == "new" else "got_it"
        state = practice.rate(card, outcome)["state"]
    return finished


def texts(client: TestClient, path: str, **params: str) -> list[str]:
    response = client.get(path, params=params)
    assert response.status_code == 200, response.text
    return [item.get("text") or item["title"] for item in response.json()["items"]]


def test_settings_hold_the_native_language_and_a_translation_choice_per_language(
    client: TestClient,
) -> None:
    settings = client.get("/api/settings").json()
    assert settings["native_language"] == "ru"
    assert settings["translation"] == {"en": False, "de": True}
    assert not {"card_language", "help_language", "learning_language"} & settings.keys()
    changed = settings | {"translation": {"en": True, "de": True}}
    assert client.put("/api/settings", json=changed).status_code == 200
    assert client.get("/api/settings").json()["translation"] == {"en": True, "de": True}


def test_a_stored_card_or_help_language_is_ignored_and_dropped_by_the_next_save(
    client: TestClient,
) -> None:
    database = client.app.state.database
    with database.write() as connection:
        connection.execute(
            "UPDATE settings SET document=json_set(document, '$.card_language', 'en', "
            "'$.help_language', 'en') WHERE id=1"
        )
    settings = client.get("/api/settings").json()
    assert settings["translation"] == {"en": False, "de": True}
    assert client.put("/api/settings", json=settings).status_code == 200
    with database.read() as connection:
        [document] = connection.execute("SELECT document FROM settings").fetchone()
    assert "card_language" not in document and "help_language" not in document


def test_a_german_package_with_russian_help_is_accepted_and_kept_german(
    practice: Practice,
) -> None:
    publish(practice.client, english(2))
    publish(practice.client, german(3))
    assert practice.rows("SELECT language, COUNT(*) AS n FROM phrases GROUP BY language") == [
        {"language": "de", "n": 3},
        {"language": "en", "n": 2},
    ]
    client = practice.client
    assert texts(client, "/api/lessons", language="de") == ["Alltag"]
    assert texts(client, "/api/lessons", language="en") == ["Meetings"]
    assert len(texts(client, "/api/phrases", language="de")) == 3
    assert texts(client, "/api/phrases", language="en", query="Tisch") == []
    assert texts(client, "/api/phrases", language="de", query="Tisch")[0].startswith("Das ist")
    assert practice.today(language="de")["new_count"] == 3
    assert practice.today(language="en")["new_count"] == 2


@pytest.mark.parametrize(
    ("package", "expected"),
    [
        (
            german(1, meaning_language="de"),
            "meaning_language must be ru: Translation in native language is on",
        ),
        (
            make_package(language="en") | {"meaning_language": "ru"},
            "meaning_language must be en: Translation in native language is off",
        ),
    ],
    ids=["german with german help", "english with russian help"],
)
def test_a_meaning_language_that_does_not_follow_settings_says_what_to_change(
    client: TestClient, package: Package, expected: str
) -> None:
    response = client.post(
        "/api/imports", json={"package": package, "references": references(package)}
    )
    assert response.status_code == 422
    assert expected in response.json()["detail"]
    assert "Translation in native language" in response.json()["detail"]
    assert client.get(f"/api/imports/{package['package_id']}").status_code == 404


def test_older_package_formats_are_rejected(client: TestClient) -> None:
    old = make_package() | {"version": 2, "card_language": "en", "help_language": "en"}
    del old["meaning_language"]
    response = client.post("/api/imports", json={"package": old, "references": references(old)})
    assert response.status_code == 422


def test_finishing_english_daily_at_its_limits_leaves_germans_allowance_whole(
    practice: Practice,
) -> None:
    publish(practice.client, english(8))
    publish(practice.client, german(8))
    practice.set_limits(due=20, new=5)
    english_cards = finish_daily(practice, "en")
    assert len(english_cards) == 5
    assert practice.daily(language="en")["card"] is None
    assert practice.today(language="en")["new_count"] == 0
    assert practice.today(language="de")["new_count"] == 5
    state = practice.daily(language="de")
    assert state["progress"] == {
        "due_completed": 0,
        "due_allowance": 20,
        "new_completed": 0,
        "new_allowance": 5,
    }
    german_cards = [state["card"], *finish_daily(practice, "de")[1:]]
    assert len(german_cards) == 5
    german_ids = {row["id"] for row in practice.rows("SELECT id FROM phrases WHERE language='de'")}
    assert {card["phrase_id"] for card in german_cards} <= german_ids
    assert {row["language"] for row in practice.log()} == {"en", "de"}


def test_extra_in_german_leaves_englishs_targets_unchanged(practice: Practice) -> None:
    publish(practice.client, english(8))
    publish(practice.client, german(8))
    practice.set_limits(due=20, new=2)
    finish_daily(practice, "en")
    finish_daily(practice, "de")
    receipt = practice.extra(language="de")["receipt"]
    assert receipt["new_target"] == 4
    assert practice.rows("SELECT language, new_target FROM extra_targets") == [
        {"language": "de", "new_target": 4}
    ]
    assert practice.daily(language="en")["progress"]["new_allowance"] == 2
    assert practice.today(language="en")["new_count"] == 0
    assert practice.today(language="de")["new_count"] >= 1


def test_each_languages_daily_resumes_its_own_view(practice: Practice) -> None:
    publish(practice.client, english(3))
    publish(practice.client, german(3))
    english_card = practice.card(language="en")
    german_card = practice.card(language="de")
    assert german_card["view_token"] != english_card["view_token"]
    assert practice.reveal(english_card)["revealed"]
    assert practice.card(language="en")["view_token"] == english_card["view_token"]
    assert practice.card(language="de")["view_token"] == german_card["view_token"]
    current = practice.client.get("/api/practice/current", params={"language": "de"}).json()
    assert current["card"]["view_token"] == german_card["view_token"]
    # Rating the English card advances English alone.
    practice.finish(english_card, "not_yet")
    assert practice.card(language="de")["view_token"] == german_card["view_token"]


def test_practising_one_german_phrase_leaves_the_english_view_alone(practice: Practice) -> None:
    publish(practice.client, english(3))
    publish(practice.client, german(3))
    english_card = practice.card(language="en")
    german_id = practice.rows("SELECT id FROM phrases WHERE language='de' LIMIT 1")[0]["id"]
    practised = practice.practise(german_id)
    practice.rate(practised, "got_it")
    assert practice.card(language="en")["view_token"] == english_card["view_token"]


def test_today_total_and_long_term_are_per_language_while_progress_counts_both(
    practice: Practice,
) -> None:
    publish(practice.client, english(4))
    publish(practice.client, german(6))
    practice.set_limits(due=20, new=2)
    finish_daily(practice, "en")
    finish_daily(practice, "de")
    english_facts, german_facts = practice.facts("en"), practice.facts("de")
    assert english_facts["today"]["new_attempts"] == 2
    assert german_facts["today"]["new_attempts"] == 2
    assert english_facts["library"]["phrase_count"] == 4
    assert german_facts["library"]["phrase_count"] == 6
    assert english_facts["work"] == german_facts["work"]
    assert english_facts["work"]["new_attempts"] == 4
    studied = practice.client.get(
        "/api/practice/studied", params={"language": "de", "learning_date": "2026-11-02"}
    ).json()["items"]
    assert len(studied) == 2 and all(item["text"].startswith("Das ist") for item in studied)


def test_a_german_only_day_extends_the_streak_and_counts_toward_the_weekly_target(
    practice: Practice,
) -> None:
    publish(practice.client, english(4))
    publish(practice.client, german(4))
    settings = practice.client.get("/api/settings").json() | {"weekly_target": 3}
    assert practice.client.put("/api/settings", json=settings).status_code == 200
    practice.rate(practice.card(language="en"), "not_yet")
    practice.later(days=1)
    practice.rate(practice.card(language="de"), "not_yet")
    for language in ("en", "de"):
        facts = practice.facts(language)
        assert facts["streak"]["current"] == 2
        assert facts["week"] == {"practice_days": 2, "target": 3}
        assert facts["work"]["practice_days"] == 2
    assert practice.facts("en")["today"]["completions"] == 0
    assert practice.facts("de")["today"]["completions"] == 1


def test_a_german_submission_snapshots_its_translation_choice_and_keeps_capitals(
    client: TestClient,
) -> None:
    def send(text: str, language: str) -> dict[str, str]:
        response = client.post(
            "/api/coach/requests", json={"text": text, "learning_language": language}
        )
        assert response.status_code == 200, response.text
        return response.json()

    morning, tomorrow = send("Morgen", "de"), send("morgen", "de")
    assert morning["outcome"] == tomorrow["outcome"] == "saved"
    assert morning["request_id"] != tomorrow["request_id"]
    assert send("Morgen", "de") == morning | {"outcome": "waiting"}
    assert send("by  and large", "en")["request_id"] == request_id("By and large", "en")
    assert send("Morgen", "en")["request_id"] != morning["request_id"]
    work = client.post("/api/imports/coach/next").json()
    assert work == {
        "package_id": morning["request_id"],
        "text": "Morgen",
        "learning_language": "de",
        "native_language": "ru",
        "translation": True,
    }


class UnusedPreparer:
    async def prepare(self, work: CoachWork) -> BasicCard:
        raise AssertionError(f"{work.text} needs no model call without a reference voice")


def test_german_coach_work_waits_for_a_voice_as_a_retryable_service_failure(
    client: TestClient,
) -> None:
    sent = client.post("/api/coach/requests", json={"text": "Tisch", "learning_language": "de"})
    package_id = sent.json()["request_id"]
    assert process_once(ClientCoachApi(client), UnusedPreparer(), PiperSpeech(None, fake=True))
    [item] = client.get("/api/coach/inbox").json()["items"]
    assert item["learning_language"] == "de" and item["outcome"] == "failed"
    assert (
        item["reason"] == "У Coach пока нет голоса для этого языка. Повторите, когда он появится."
    )
    [notice] = client.get("/api/practice/facts", params={"language": "en"}).json()["notices"]
    assert notice["package_id"] == package_id and notice["retry_helps"]
    retried = client.post(f"/api/coach/requests/{package_id}/retry").json()
    assert retried == {"request_id": package_id, "outcome": "saved"}


def test_a_published_german_coach_card_is_german(client: TestClient) -> None:
    sent = client.post("/api/coach/requests", json={"text": "Tisch", "learning_language": "de"})
    package_id = sent.json()["request_id"]
    client.post("/api/imports/coach/next")
    card = {"target": "Tisch", "text": "Tisch", "cue": "T__h", "instruction": "Скажите слово."}
    response = client.post(
        f"/api/imports/{package_id}/result",
        json={"result_id": package_id, "basic": card, "reference": silent_reference()},
    )
    assert response.status_code == 200, response.text
    assert texts(client, "/api/phrases", language="de") == ["Tisch"]
    assert texts(client, "/api/phrases", language="en") == []
    assert texts(client, "/api/lessons", language="de") == ["Added with Coach"]


def test_a_german_session_keeps_an_english_skip_gradable(practice: Practice) -> None:
    publish(practice.client, english(3))
    publish(practice.client, german(3))
    card = practice.reveal(practice.card(language="en"))
    skipped = practice.finish(card, "skipped")["receipt"]
    practice.card(language="de")
    graded = practice.client.post(
        "/api/practice/grade-skipped",
        json={"completion_id": skipped["completion_id"], "rating": "got_it"},
    )
    assert graded.status_code == 200, graded.text
    assert graded.json()["progress"]["new_completed"] == 1
    assert practice.daily(language="de")["progress"]["new_completed"] == 0
