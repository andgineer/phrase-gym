"""Preparation leases and durable source handoff receipts."""

from sqlalchemy import Text
from sqlalchemy.orm import Mapped, mapped_column

from phrase_gym.tables import Base


class PreparationRow(Base):
    __tablename__ = "preparation_job"
    id: Mapped[str] = mapped_column(primary_key=True)
    input_hash: Mapped[str] = mapped_column(unique=True)
    payload: Mapped[str] = mapped_column(Text)
    base_bindings: Mapped[str] = mapped_column(Text)
    status: Mapped[str]
    attempts: Mapped[int]
    lease_token: Mapped[str | None]
    lease_until: Mapped[str | None]
    retry_at: Mapped[str]
    error: Mapped[str | None]
    result: Mapped[str | None] = mapped_column(Text)
    review: Mapped[str | None] = mapped_column(Text)
    published_import_id: Mapped[str | None]


class HandoffRow(Base):
    __tablename__ = "source_handoff"
    export_id: Mapped[str] = mapped_column(primary_key=True)
    payload_hash: Mapped[str]
    payload: Mapped[str] = mapped_column(Text)
    preparation_id: Mapped[str]
