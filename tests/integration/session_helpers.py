from uuid import uuid4

from tests.integration.helpers import phrases, publish, upload


def ready_package(client, package):
    for target in package["targets"]:
        response = upload(client, text=target["text"])
        assert response.status_code == 200, response.text
        target["reference_audio_id"] = response.json()["id"]
    publish(client, package)
    return phrases(client)


def start(client, **parameters):
    return client.post(
        "/api/v1/sessions", json={"id": str(uuid4()), "language": "en", **parameters}
    )
