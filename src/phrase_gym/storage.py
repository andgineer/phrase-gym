"""Durable SQLite tables and short, serialized write transactions."""

from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

from alembic import command
from alembic.config import Config as AlembicConfig
from sqlalchemy import create_engine, event
from sqlalchemy.orm import Session, sessionmaker


class Database:
    def __init__(self, data_dir: Path):
        data_dir.mkdir(parents=True, exist_ok=True)
        self.engine = create_engine(
            f"sqlite:///{(data_dir / 'phrase-gym.sqlite3').as_posix()}",
            connect_args={"check_same_thread": False},
        )
        event.listen(self.engine, "connect", self.configure_connection)
        event.listen(self.engine, "begin", self.begin_transaction)
        self.sessions = sessionmaker(self.engine, expire_on_commit=False)

    @staticmethod
    def configure_connection(connection, _record) -> None:
        connection.isolation_level = None
        cursor = connection.cursor()
        cursor.execute("PRAGMA foreign_keys=ON")
        cursor.execute("PRAGMA journal_mode=WAL")
        cursor.execute("PRAGMA busy_timeout=5000")
        cursor.close()

    @staticmethod
    def begin_transaction(connection) -> None:
        mode = "BEGIN IMMEDIATE" if connection.get_execution_options().get("write") else "BEGIN"
        connection.exec_driver_sql(mode)

    def migrate(self) -> None:
        config = AlembicConfig()
        config.set_main_option("script_location", str(Path(__file__).parent / "migrations"))
        with self.engine.begin() as connection:
            config.attributes["connection"] = connection
            command.upgrade(config, "head")

    @contextmanager
    def read(self) -> Iterator[Session]:
        with self.sessions() as session:
            yield session

    @contextmanager
    def write(self) -> Iterator[Session]:
        with (
            self.engine.connect().execution_options(write=True) as connection,
            connection.begin(),
            Session(bind=connection, expire_on_commit=False) as session,
        ):
            yield session
            session.flush()

    def close(self) -> None:
        self.engine.dispose()
