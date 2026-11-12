from fastapi.testclient import TestClient

TYPED = "by and large"


def fail_preparation(client: TestClient, error: str) -> str:
    work = client.post("/api/imports/coach/next").json()
    client.post(f"/api/imports/{work['package_id']}/failure", json={"error": error})
    return work["package_id"]


def notices(client: TestClient) -> list[dict[str, str]]:
    return client.get("/api/practice/facts").json()["notices"]


def test_failure_notice_arrives_with_practice_facts_until_dismissed(client: TestClient) -> None:
    assert notices(client) == []
    package_id = client.post("/api/coach/requests", json={"text": TYPED}).json()["request_id"]
    assert notices(client) == []
    fail_preparation(client, "Codex is not logged in")
    assert notices(client) == [
        {
            "package_id": package_id,
            "kind": "coach",
            "label": TYPED,
            "error": "Codex is not logged in",
            "retry_helps": True,
        }
    ]
    assert client.delete(f"/api/imports/{package_id}/notice").status_code == 200
    assert notices(client) == []
    client.post(f"/api/imports/{package_id}/failure", json={"error": "Codex is not logged in"})
    assert notices(client) == []
    status = client.get(f"/api/imports/{package_id}").json()
    assert (status["status"], status["error"]) == ("error", "Codex is not logged in")


def test_resubmission_clears_the_notice_and_a_new_failure_shows_again(client: TestClient) -> None:
    client.post("/api/coach/requests", json={"text": TYPED})
    package_id = fail_preparation(client, "Piper unavailable")
    resubmitted = client.post("/api/coach/requests", json={"text": TYPED}).json()
    assert resubmitted == {"request_id": package_id, "outcome": "saved"}
    assert client.get(f"/api/imports/{package_id}").json() == {
        "package_id": package_id,
        "status": "pending",
        "error": None,
        "added_count": None,
    }
    assert notices(client) == []
    fail_preparation(client, "The input is not English")
    assert [notice["error"] for notice in notices(client)] == ["The input is not English"]


def test_dismissing_an_absent_notice_changes_nothing(client: TestClient) -> None:
    assert client.delete("/api/imports/unknown-package/notice").status_code == 200
    assert notices(client) == []


def test_a_failure_retyping_cannot_fix_says_so_and_a_resubmission_resets_it(
    client: TestClient,
) -> None:
    package_id = client.post("/api/coach/requests", json={"text": TYPED}).json()["request_id"]
    client.post("/api/imports/coach/next")
    client.post(
        f"/api/imports/{package_id}/failure",
        json={"error": "No English word or expression was found.", "retry_helps": False},
    )
    assert [notice["retry_helps"] for notice in notices(client)] == [False]
    client.post("/api/coach/requests", json={"text": TYPED})
    fail_preparation(client, "Claude reported a usage limit")
    assert [notice["retry_helps"] for notice in notices(client)] == [True]
