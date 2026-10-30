from uuid import uuid4

import pytest
from fastapi.testclient import TestClient

from phrase_gym.server.app import create_app
from tests.phrase_packages import Package, lesson, make_package, phrase, publish, upload_take
from tests.practice_client import Practice


@pytest.fixture
def practice(client: TestClient) -> Practice:
    return Practice(client)


def many(count: int) -> Package:
    return make_package(lessons=[lesson("unit-1", "Meetings", [phrase(i) for i in range(count)])])


def ids(practice: Practice) -> list[str]:
    return [row["id"] for row in practice.rows("SELECT id FROM phrases ORDER BY position")]


def completed_today(practice: Practice, due: int, new: int) -> None:
    rows = [("recall", "got_it")] * due + [("first_study", None)] * new
    with practice.database.write() as connection:
        connection.executemany(
            "INSERT INTO practice_log (id, phrase_id, phrase_text, lesson_title, completed_at, "
            "learning_date, kind, rating, skipped, fsrs_updated, mode) "
            "VALUES (?, NULL, 'Earlier phrase.', 'Earlier', ?, '2026-10-28', ?, ?, 0, 0, 'daily')",
            [(str(uuid4()), practice.now.isoformat(), kind, rating) for kind, rating in rows],
        )


def targets(result: dict) -> tuple[int, int]:
    return result["receipt"]["due_target"], result["receipt"]["new_target"]


@pytest.mark.parametrize(
    ("counts", "expected"),
    [((12, 2), (20, 5)), ((20, 5), (40, 10)), ((27, 7), (47, 12))],
)
def test_extra_targets_follow_limits_and_todays_completions(
    practice: Practice, counts: tuple[int, int], expected: tuple[int, int]
) -> None:
    completed_today(practice, *counts)
    result = practice.extra()
    assert result["replayed"] is False and targets(result) == expected
    assert result["state"]["progress"] == {
        "due_completed": counts[0],
        "due_allowance": expected[0],
        "new_completed": counts[1],
        "new_allowance": expected[1],
    }
    assert practice.rows("SELECT * FROM extra_targets") == [
        {"learning_date": "2026-10-28", "due_target": expected[0], "new_target": expected[1]}
    ]


def test_repeated_presses_do_not_compound_or_reset_completed_work(practice: Practice) -> None:
    completed_today(practice, 20, 5)
    first = practice.extra()
    second = practice.extra()
    assert targets(first) == targets(second) == (40, 10)
    assert second["state"]["progress"]["due_completed"] == 20
    assert len(practice.log()) == 25
    completed_today(practice, 21, 0)
    assert targets(practice.extra()) == (61, 10)


def test_lower_settings_keep_earlier_targets_and_zero_disables_a_category(
    practice: Practice,
) -> None:
    completed_today(practice, 20, 5)
    practice.extra()
    practice.set_limits(due=10, new=2)
    lowered = practice.extra()
    assert targets(lowered) == (40, 10)
    assert lowered["state"]["progress"]["due_allowance"] == 40
    practice.set_limits(due=10, new=0)
    disabled = practice.extra()
    assert targets(disabled) == (40, 10)
    assert disabled["state"]["progress"]["new_allowance"] == 0
    practice.later(days=1)
    assert targets(practice.extra()) == (10, 0)


def test_zero_category_without_earlier_target_gets_no_extension(practice: Practice) -> None:
    publish(practice.client, many(3))
    practice.set_limits(due=20, new=0)
    result = practice.extra()
    assert targets(result) == (20, 0)
    assert result["state"]["card"] is None
    assert result["state"]["message"] == "Nothing waiting today."


def test_extra_opens_a_fresh_traversal_that_offers_skipped_due_again(practice: Practice) -> None:
    publish(practice.client, many(2))
    first_id, second_id = ids(practice)
    practice.make_due(first_id, second_id)
    first = practice.card()
    second = practice.finish(first, "skipped")["state"]["card"]
    practice.reveal(second)
    assert practice.finish(second, "got_it")["state"]["card"] is None
    extra = practice.extra()["state"]
    again = extra["card"]
    assert again["phrase_id"] == first_id and again["mode"] == "extra"
    assert again["view_token"] not in (first["view_token"], second["view_token"])
    assert extra["progress"]["due_completed"] == 2
    assert practice.card() == again
    practice.reveal(again)
    receipt = practice.finish(again, "effort")["receipt"]
    assert (receipt["mode"], receipt["fsrs_updated"]) == ("extra", True)
    assert practice.daily()["message"] == "Nothing waiting today. Next due: 2026-10-29."


def test_extra_never_cycles_not_due_phrases_and_explains_immediately(practice: Practice) -> None:
    publish(practice.client, many(2))
    practice.study_all_new()
    result = practice.extra()["state"]
    assert result["card"] is None
    assert result["message"] == "Nothing waiting today. Next due: 2026-10-29."
    assert len(practice.log()) == 2


def test_extra_continues_after_the_base_limits_and_ends_with_a_message(
    practice: Practice,
) -> None:
    publish(practice.client, many(4))
    practice.set_limits(due=1, new=1)
    practice.study_all_new()
    state = practice.extra()["state"]
    studied = []
    while state["card"]:
        assert state["card"]["mode"] == "extra"
        studied.append(state["card"]["phrase_id"])
        state = practice.finish(state["card"], "studied")["state"]
    assert len(studied) == 1
    assert state["progress"]["new_allowance"] == 2
    assert state["message"] == "Today's Extra practice is complete."
    modes = [row["mode"] for row in practice.log()]
    assert modes == ["daily", "extra"]


def test_extra_replaces_the_open_view_so_the_old_card_is_stale(practice: Practice) -> None:
    publish(practice.client, many(3))
    old = practice.card()
    practice.reveal(old)
    fresh = practice.extra()["state"]["card"]
    assert fresh["view_token"] != old["view_token"]
    assert practice.complete(old, "studied").status_code == 409
    assert practice.action("reveal", old).status_code == 409
    assert practice.log() == []
    assert practice.card() == fresh


def test_lost_extra_response_retry_never_extends_again_or_replaces_newer_views(
    practice: Practice, monkeypatch: pytest.MonkeyPatch
) -> None:
    publish(practice.client, many(12))
    practice.set_limits(due=20, new=2)
    practice.study_all_new()
    operation = str(uuid4())
    original = practice.extra(operation_id=operation)
    assert targets(original) == (20, 4)
    state = practice.finish(original["state"]["card"], "studied")["state"]
    practice.finish(state["card"], "studied")
    replay = practice.extra(operation_id=operation)
    assert replay == {"receipt": original["receipt"], "replayed": True, "state": None}
    assert practice.daily()["progress"]["new_allowance"] == 4
    newer = practice.extra()["state"]["card"]
    assert practice.extra(operation_id=operation)["replayed"] is True
    assert practice.card() == newer
    assert practice.rows("SELECT new_target FROM extra_targets") == [{"new_target": 6}]
    practice.later(days=1)
    assert practice.extra(operation_id=operation)["receipt"] == original["receipt"]
    assert practice.rows("SELECT learning_date FROM extra_targets") == [
        {"learning_date": "2026-10-28"}
    ]
    with TestClient(create_app()) as restarted:
        retried = Practice(restarted, practice.now)
        assert retried.extra(operation_id=operation)["receipt"] == original["receipt"]
        assert len(retried.log()) == 4


def test_changed_payload_or_another_action_under_one_id_conflicts(practice: Practice) -> None:
    publish(
        practice.client,
        make_package(lessons=[lesson("a", "A", [phrase(1)]), lesson("b", "B", [phrase(2)])]),
    )
    lesson_id = practice.client.get("/api/lessons").json()["items"][0]["id"]
    operation = str(uuid4())
    card = practice.extra(operation_id=operation)["state"]["card"]
    assert practice.press_extra(lesson_id, operation).status_code == 409
    completion = str(uuid4())
    practice.finish(card, "studied", completion)
    assert practice.press_extra(operation_id=completion).status_code == 409
    assert practice.complete(practice.card(), "studied", operation).status_code == 409
    assert practice.press_extra("missing-lesson").status_code == 404
    assert len(practice.log()) == 1


def test_late_takes_upload_after_the_extra_view_moves_on(practice: Practice) -> None:
    publish(practice.client, many(3))
    card = practice.extra()["state"]["card"]
    practice.extra()
    progress = practice.daily()["progress"]
    log = practice.log()
    take = str(uuid4())
    assert upload_take(practice.client, card, take).status_code == 200
    assert practice.daily()["progress"] == progress and practice.log() == log
    history = practice.client.get(f"/api/phrases/{card['phrase_id']}/recordings").json()
    assert [(item["id"], item["mode"]) for item in history["items"]] == [(take, "extra")]
