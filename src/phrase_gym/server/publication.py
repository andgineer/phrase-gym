import json
import sqlite3
from datetime import UTC, datetime
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

from phrase_gym.server.contracts import PhrasePackage
from phrase_gym.server.storage import write_private


def card_identity(source_id: str, item_id: str) -> str:
    encoded = json.dumps([source_id, item_id], separators=(",", ":"))
    return str(uuid5(NAMESPACE_URL, encoded))


def publish_batch(
    connection: sqlite3.Connection,
    data_dir: Path,
    package: PhrasePackage,
    decoded: dict[str, bytes],
    result: tuple[str, str],
) -> None:
    result_id, result_hash = result
    references = data_dir / "references"
    references.mkdir(mode=0o700, exist_ok=True)
    references.chmod(0o700)
    created: list[Path] = []
    try:
        for card in package.cards:
            card_id = card_identity(package.source_id, card.item_id)
            destination = references / f"{card_id}.wav"
            if destination.exists():
                destination.unlink()
            created.append(destination)
            write_private(destination, decoded[card.item_id])
            connection.execute(
                "INSERT INTO cards (id,source_id,item_id,source_label,source_link,answer,"
                "explanation,cues_json,variants_json,detailed_explanation,"
                "reference_path,created_at,studied_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)",
                (
                    card_id,
                    package.source_id,
                    card.item_id,
                    package.source_label,
                    str(package.source_link) if package.source_link else None,
                    card.answer,
                    card.explanation,
                    json.dumps([cue.model_dump() for cue in card.cues]),
                    json.dumps(card.accepted_variants),
                    card.detailed_explanation,
                    str(destination),
                    datetime.now(UTC).isoformat(),
                ),
            )
        connection.execute(
            "UPDATE imports SET status='added', error=NULL, added_count=?, "
            "result_id=?, result_hash=? WHERE package_id=?",
            (len(package.cards), result_id, result_hash, package.package_id),
        )
    except BaseException:
        for path in created:
            path.unlink(missing_ok=True)
        raise
