from fastapi.testclient import TestClient

from phrase_gym.__about__ import __version__


def test_health_reports_installed_version_in_the_generated_contract(client: TestClient) -> None:
    assert client.get("/api/health").json() == {"status": "ok", "version": __version__}
    document = client.get("/openapi.json").json()
    assert document["info"]["version"] == __version__
    assert document["paths"]["/api/health"]["get"]["responses"]["200"]["content"][
        "application/json"
    ]["schema"] == {"$ref": "#/components/schemas/Health"}
    assert document["components"]["schemas"]["Health"]["required"] == ["status", "version"]
