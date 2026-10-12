import json
from uuid import uuid5

from phrase_gym.contracts.media import MediaMetadata
from phrase_gym.media.service import MEDIA_NAMESPACE
from phrase_gym.storage import Database, MediaRow
from tests.integration.helpers import upload


def test_settings_key_order_reuses_asset_and_real_changes_do_not(client):
    first = upload(client, settings={"speed": 1, "processing": False}).json()
    reordered = upload(client, settings={"processing": False, "speed": 1}).json()
    assert reordered == first
    changed = upload(client, settings={"processing": True, "speed": 1}).json()
    assert changed["id"] != first["id"]


def test_upload_retry_preserves_legacy_asset_id_and_repairs_bytes(client, config):
    original = upload(client, settings={"speed": 1, "processing": False}).json()
    metadata = MediaMetadata.model_validate(original["metadata"])
    metadata.settings = {"speed": 1, "processing": False}
    legacy_json = metadata.model_dump_json()
    legacy_id = str(
        uuid5(MEDIA_NAMESPACE, "|".join((original["sha256"], "audio/wav", legacy_json)))
    )
    database = Database(config.data_dir)
    with database.write() as session:
        row = session.get(MediaRow, original["id"])
        row.id = legacy_id
        row.metadata_json = legacy_json
    database.close()
    (config.data_dir / "media" / original["sha256"]).unlink()
    recovered = upload(client, settings={"processing": False, "speed": 1}).json()
    assert recovered["id"] == legacy_id
    assert recovered["available"]
    assert recovered["metadata"] == json.loads(legacy_json)
