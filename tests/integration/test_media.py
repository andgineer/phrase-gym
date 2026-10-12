import os
import time
import pytest
from tests.integration.helpers import publish, phrases, wav_bytes, upload


def test_media_original_bytes_range_retry_recovery(client, config, package):
    asset = upload(client).json()
    assert asset["available"]
    assert upload(client).json()["id"] == asset["id"]
    different_voice = upload(client, performer="Another performer").json()
    assert different_voice["id"] != asset["id"]
    path = f"/api/v1/media/{asset['id']}"
    response = client.get(path, headers={"Range": "bytes=0-43"})
    assert response.status_code == 206
    assert response.content == wav_bytes()[:44]
    assert response.headers["content-type"] == "audio/wav"
    assert client.get(path).content == wav_bytes()
    assert client.get(path, headers={"Range": "bytes=999999-"}).status_code == 416
    package["targets"][0]["reference_audio_id"] = asset["id"]
    publish(client, package)
    assert phrases(client, q="greetings")[0]["audio_status"] == "ready"
    (config.data_dir / "media" / asset["sha256"]).unlink()
    assert client.get(path).status_code == 503
    assert phrases(client, q="greetings")[0]["audio_status"] == "missing"
    assert asset["id"] in client.post("/api/v1/media/reconcile").json()["missing_ids"]
    upload(client)
    assert client.get(path).content == wav_bytes()


def test_audio_cannot_use_word_only_stale_language_or_leaking_cue(client, package):
    for overrides in ({"text": "day"}, {"language": "sr", "locale": "sr-RS"}, {"role": "cue"}):
        asset = upload(client, **overrides).json()
        package["targets"][0]["reference_audio_id"] = asset["id"]
        assert client.post("/api/v1/imports", json=package).json()["status"] == "rejected"
    package["targets"][0]["reference_audio_id"] = None
    asset = upload(client, role="cue").json()
    package["targets"][0]["cues"][0]["audio_id"] = asset["id"]
    assert client.post("/api/v1/imports", json=package).json()["status"] == "rejected"
    asset = upload(client, role="cue", text="your day been so far?").json()
    package["targets"][0]["cues"][0]["audio_id"] = asset["id"]
    assert client.post("/api/v1/imports", json=package).json()["status"] == "staged"


@pytest.mark.parametrize("audio", [b"", b"not-audio", wav_bytes()[:-8]])
def test_invalid_audio_rejected(client, audio):
    assert upload(client, audio=audio).status_code == 422


def test_orphan_reconciliation_never_deletes_original_media(client, config):
    original = upload(client).json()
    orphan = config.data_dir / "media" / ("0" * 64)
    orphan.write_bytes(b"orphan")
    interrupted = config.data_dir / "staging" / "interrupted.part"
    interrupted.write_bytes(b"partial")
    os.utime(interrupted, (time.time() - 7200, time.time() - 7200))
    active = config.data_dir / "staging" / "active.part"
    active.write_bytes(b"active upload")
    report = client.post("/api/v1/media/reconcile").json()
    assert report["orphan_hashes"] == [orphan.name]
    assert report["removed_staging_files"] == 1
    assert active.exists() and orphan.exists()
    assert client.get(f"/api/v1/media/{original['id']}").content == wav_bytes()
