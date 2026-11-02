import os
import sqlite3
from collections.abc import Iterator
from contextlib import closing
from datetime import UTC, datetime
from importlib.resources import files
from pathlib import Path


def _load_steps() -> dict[int, str]:
    folder = files("phrase_gym.server").joinpath("schema")
    return {
        int(item.name.removesuffix(".sql")): item.read_text(encoding="utf-8")
        for item in folder.iterdir()
        if item.name.endswith(".sql")
    }


STEPS = _load_steps()
# Databases created before step files existed start at this version; nothing older upgrades.
BASELINE_VERSION = min(STEPS)
SCHEMA_VERSION = str(max(STEPS))
UPGRADABLE_VERSIONS = {str(version) for version in range(BASELINE_VERSION, max(STEPS) + 1)}


def statements(script: str) -> Iterator[str]:
    buffer = ""
    for part in script.split(";"):
        buffer += part + ";"
        if sqlite3.complete_statement(buffer):
            if buffer.strip(" \n;"):
                yield buffer
            buffer = ""


def apply_steps(connection: sqlite3.Connection, current: str | None) -> None:
    first = BASELINE_VERSION if current is None else int(current) + 1
    for version in range(first, max(STEPS) + 1):
        for statement in statements(STEPS[version]):
            connection.execute(statement)
    if current is None:
        connection.execute("INSERT INTO meta VALUES ('schema_version', ?)", (SCHEMA_VERSION,))
    else:
        connection.execute(
            "UPDATE meta SET value=? WHERE key='schema_version'",
            (SCHEMA_VERSION,),
        )


def backup_before_upgrade(path: Path, version: str) -> Path:
    stamp = datetime.now(UTC).strftime("%Y%m%d-%H%M%S-%f")
    target = path.with_name(f"{path.stem}-schema-{version}-{stamp}.sqlite3")
    os.close(os.open(target, os.O_CREAT | os.O_WRONLY | os.O_EXCL, 0o600))
    with (
        closing(sqlite3.connect(path)) as source,
        closing(sqlite3.connect(target)) as copy,
    ):
        source.backup(copy)
        # The copy stays one self-contained file rather than inheriting WAL sidecars.
        copy.execute("PRAGMA journal_mode=DELETE")
    return target
