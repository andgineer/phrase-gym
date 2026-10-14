"""Explicit local runtime configuration; never expose provider environment variables."""

import os
from dataclasses import dataclass
from pathlib import Path

from platformdirs import user_data_path


@dataclass(frozen=True)
class Config:
    data_dir: Path
    client_dir: Path | None = None
    preparation_config: Path | None = None
    allowed_hosts: tuple[str, ...] = ("127.0.0.1", "localhost", "testserver")
    allowed_origins: tuple[str, ...] = ()

    @classmethod
    def from_env(cls) -> "Config":
        client = os.environ.get("PHRASE_GYM_CLIENT_DIR")
        bundled = Path(__file__).parent / "static"
        client_dir = Path(client) if client else bundled if bundled.is_dir() else None
        return cls(
            data_dir=Path(os.environ.get("PHRASE_GYM_DATA_DIR", user_data_path("phrase-gym"))),
            client_dir=client_dir,
            preparation_config=Path(os.environ["PHRASE_GYM_PREPARATION_CONFIG"])
            if os.environ.get("PHRASE_GYM_PREPARATION_CONFIG")
            else None,
            allowed_hosts=tuple(
                os.environ.get("PHRASE_GYM_HOSTS", "127.0.0.1,localhost").split(","),
            ),
            allowed_origins=tuple(
                filter(None, os.environ.get("PHRASE_GYM_ORIGINS", "").split(",")),
            ),
        )
