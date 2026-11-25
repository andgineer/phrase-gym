import json

from fastapi.testclient import TestClient

from phrase_gym.server.coach_requests import request_id

TYPED = "fom"


def claimed(client: TestClient, text: str = TYPED) -> str:
    package_id = client.post("/api/coach/requests", json={"text": text}).json()["request_id"]
    assert client.post("/api/imports/coach/next").json()["package_id"] == package_id
    return package_id


def fail(client: TestClient, package_id: str, error: str, retry_helps: bool) -> None:
    response = client.post(
        f"/api/imports/{package_id}/failure", json={"error": error, "retry_helps": retry_helps}
    )
    assert response.status_code == 200


def inbox(client: TestClient) -> list[dict[str, object]]:
    return client.get("/api/coach/inbox").json()["items"]


def notices(client: TestClient) -> list[str]:
    return [notice["label"] for notice in client.get("/api/practice/facts").json()["notices"]]


REASON = "“fom” could be several English words, such as from or form."


def test_a_rejection_shows_original_language_reason_and_stays_out_of_practice(
    client: TestClient,
) -> None:
    package_id = claimed(client)
    fail(client, package_id, REASON, retry_helps=False)
    assert inbox(client) == [
        {
            "request_id": package_id,
            "text": TYPED,
            "learning_language": "en",
            "outcome": "rejected",
            "reason": REASON,
            "correction": None,
        }
    ]
    assert notices(client) == []
    assert client.get("/api/lessons").json()["all_lessons"]["phrase_count"] == 0


def test_repeating_rejected_text_returns_its_result_without_restarting_work(
    client: TestClient,
) -> None:
    package_id = claimed(client)
    fail(client, package_id, REASON, retry_helps=False)
    assert client.post(f"/api/coach/requests/{package_id}/dismiss").status_code == 200
    assert inbox(client) == []
    again = client.post("/api/coach/requests", json={"text": f"  {TYPED.upper()} "}).json()
    assert again == {"request_id": package_id, "outcome": "rejected"}
    assert client.post("/api/imports/coach/next").json() is None
    assert [item["reason"] for item in inbox(client)] == [REASON]
    retry = client.post(f"/api/coach/requests/{package_id}/retry")
    assert retry.status_code == 409


def test_dismissal_persists_and_keeps_the_recorded_result(client: TestClient) -> None:
    package_id = claimed(client)
    fail(client, package_id, REASON, retry_helps=False)
    client.post(f"/api/coach/requests/{package_id}/dismiss")
    assert inbox(client) == []
    status = client.get(f"/api/imports/{package_id}").json()
    assert (status["status"], status["error"]) == ("error", REASON)
    assert client.post("/api/imports/coach/next").json() is None
    assert client.post("/api/coach/requests/unknown/dismiss").status_code == 404


def test_a_service_failure_is_retryable_and_announced_in_practice(client: TestClient) -> None:
    package_id = claimed(client, "by and large")
    fail(client, package_id, "Gemini: Not logged into Antigravity — run: agy login", True)
    [item] = inbox(client)
    assert (item["outcome"], item["reason"]) == (
        "failed",
        "Gemini: Not logged into Antigravity — run: agy login",
    )
    assert notices(client) == ["by and large"]
    retried = client.post(f"/api/coach/requests/{package_id}/retry").json()
    assert retried == {"request_id": package_id, "outcome": "saved"}
    assert (inbox(client), notices(client)) == ([], [])
    assert client.post("/api/imports/coach/next").json()["package_id"] == package_id
    assert client.post("/api/coach/requests/unknown/retry").status_code == 404


def test_resubmitting_after_a_service_failure_retries_it_and_clears_the_notice(
    client: TestClient,
) -> None:
    package_id = claimed(client, "by and large")
    fail(client, package_id, "Gemini: agent timed out", True)
    again = client.post("/api/coach/requests", json={"text": "by and large"}).json()
    assert again == {"request_id": package_id, "outcome": "saved"}
    assert (inbox(client), notices(client)) == ([], [])


def test_dismissing_a_failure_in_coach_also_clears_its_practice_notice(client: TestClient) -> None:
    package_id = claimed(client, "by and large")
    fail(client, package_id, "Gemini: agent timed out", True)
    client.post(f"/api/coach/requests/{package_id}/dismiss")
    assert (inbox(client), notices(client)) == ([], [])


def test_submissions_capture_the_active_learning_language(client: TestClient) -> None:
    assert client.get("/api/settings").json()["learning_language"] == "en"
    package_id = client.post("/api/coach/requests", json={"text": "by and large"}).json()[
        "request_id"
    ]
    # English requests keep the identity they had before the language was recorded.
    assert package_id == request_id("by and large")
    assert request_id("by and large", "de") != package_id
    with client.app.state.database.read() as connection:
        payload = connection.execute(
            "SELECT payload FROM imports WHERE package_id=?", (package_id,)
        ).fetchone()[0]
    assert json.loads(payload) == {"text": "by and large", "learning_language": "en"}
    assert client.post("/api/imports/coach/next").json()["learning_language"] == "en"


def test_a_request_saved_before_languages_were_recorded_is_english(client: TestClient) -> None:
    with client.app.state.database.write() as connection:
        connection.execute(
            "INSERT INTO imports (package_id, content_hash, payload, status, created_at, kind) "
            "VALUES ('coach-old', 'h', '{\"text\": \"by and large\"}', 'pending', "
            "'2026-11-01T10:00:00+00:00', 'coach')"
        )
    work = client.post("/api/imports/coach/next").json()
    assert work == {"package_id": "coach-old", "text": "by and large", "learning_language": "en"}
