from fastapi.testclient import TestClient

from tests.phrase_packages import Package


def fail_preparation(client: TestClient, package_id: str, error: str) -> None:
    assert client.post("/api/imports/coach/next").json()["package_id"] == package_id
    client.post(f"/api/imports/{package_id}/failure", json={"error": error})


def notices(client: TestClient) -> list[dict[str, str]]:
    return client.get("/api/practice/facts").json()["notices"]


def test_failure_notice_arrives_with_practice_facts_until_dismissed(
    client: TestClient, package: Package
) -> None:
    package_id = package["package_id"]
    assert notices(client) == []
    client.post("/api/imports", json={"package": package})
    assert notices(client) == []
    fail_preparation(client, package_id, "Piper unavailable")
    assert notices(client) == [
        {
            "package_id": package_id,
            "source_label": package["source_label"],
            "error": "Piper unavailable",
        }
    ]
    assert client.delete(f"/api/imports/{package_id}/notice").status_code == 200
    assert notices(client) == []
    client.post(f"/api/imports/{package_id}/failure", json={"error": "Piper unavailable"})
    assert notices(client) == []
    status = client.get(f"/api/imports/{package_id}").json()
    assert (status["status"], status["error"]) == ("error", "Piper unavailable")


def test_resubmission_clears_the_notice_and_a_new_failure_shows_again(
    client: TestClient, package: Package
) -> None:
    package_id = package["package_id"]
    client.post("/api/imports", json={"package": package})
    fail_preparation(client, package_id, "Piper unavailable")
    assert client.post("/api/imports", json={"package": package}).json()["status"] == "pending"
    assert notices(client) == []
    fail_preparation(client, package_id, "Voice file missing")
    assert [notice["error"] for notice in notices(client)] == ["Voice file missing"]


def test_dismissing_an_absent_notice_changes_nothing(client: TestClient) -> None:
    assert client.delete("/api/imports/unknown-package/notice").status_code == 200
    assert notices(client) == []
