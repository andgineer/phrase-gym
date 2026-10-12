"""Foundation content and session persistence models."""

from sqlalchemy import CheckConstraint, ForeignKey, Text, UniqueConstraint
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column


class Base(DeclarativeBase):
    pass


class AppSettings(Base):
    __tablename__ = "app_settings"
    id: Mapped[int] = mapped_column(primary_key=True)
    payload: Mapped[str] = mapped_column(Text)


class Target(Base):
    __tablename__ = "target"
    id: Mapped[str] = mapped_column(primary_key=True)
    language: Mapped[str]
    kind: Mapped[str]
    meaning_key: Mapped[str]
    recall_expression: Mapped[str] = mapped_column(Text)
    current_revision: Mapped[int]


class TargetRevision(Base):
    __tablename__ = "target_revision"
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"), primary_key=True)
    revision: Mapped[int] = mapped_column(primary_key=True)
    payload: Mapped[str] = mapped_column(Text)
    fingerprint: Mapped[str]
    created_at: Mapped[str]


class SourceItem(Base):
    __tablename__ = "source_item"
    namespace: Mapped[str] = mapped_column(primary_key=True)
    source_id: Mapped[str] = mapped_column(primary_key=True)
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"))
    source_revision: Mapped[str]


class LessonRow(Base):
    __tablename__ = "lesson"
    __table_args__ = (UniqueConstraint("namespace", "source_id"),)
    id: Mapped[str] = mapped_column(primary_key=True)
    namespace: Mapped[str]
    source_id: Mapped[str]
    language: Mapped[str]
    title: Mapped[str] = mapped_column(Text)


class Membership(Base):
    __tablename__ = "lesson_membership"
    lesson_id: Mapped[str] = mapped_column(ForeignKey("lesson.id"), primary_key=True)
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"), primary_key=True)
    position: Mapped[int]


class DialogueRow(Base):
    __tablename__ = "dialogue"
    __table_args__ = (UniqueConstraint("namespace", "source_id", "revision"),)
    id: Mapped[str] = mapped_column(primary_key=True)
    namespace: Mapped[str]
    source_id: Mapped[str]
    revision: Mapped[int] = mapped_column(primary_key=True)
    language: Mapped[str]
    payload: Mapped[str] = mapped_column(Text)
    target_ids: Mapped[str] = mapped_column(Text)


class ImportRow(Base):
    __tablename__ = "content_import"
    id: Mapped[str] = mapped_column(primary_key=True)
    payload: Mapped[str] = mapped_column(Text)
    report: Mapped[str] = mapped_column(Text)
    status: Mapped[str]
    created_at: Mapped[str]


class MediaRow(Base):
    __tablename__ = "media"
    id: Mapped[str] = mapped_column(primary_key=True)
    sha256: Mapped[str]
    byte_length: Mapped[int]
    mime_type: Mapped[str]
    metadata_json: Mapped[str] = mapped_column(Text)


class ExposureRow(Base):
    __tablename__ = "exposure"
    id: Mapped[str] = mapped_column(primary_key=True)
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"))
    revision: Mapped[int]
    payload: Mapped[str] = mapped_column(Text)
    received_at: Mapped[str]


class ScheduleRow(Base):
    __tablename__ = "target_schedule"
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"), primary_key=True)
    version: Mapped[int]
    due_at: Mapped[str]
    memory_state: Mapped[str | None] = mapped_column(Text)
    policy_version: Mapped[str | None]


class SessionRow(Base):
    __tablename__ = "practice_session"
    __table_args__ = (
        UniqueConstraint("open_slot", name="one_open_practice_session"),
        CheckConstraint(
            "(status IN ('active', 'paused') AND open_slot IS 1) OR "
            "(status IN ('superseded', 'completed') AND open_slot IS NULL)",
            name="session_slot_matches_status",
        ),
    )
    id: Mapped[str] = mapped_column(primary_key=True)
    start_payload: Mapped[str] = mapped_column(Text)
    manifest: Mapped[str] = mapped_column(Text)
    status: Mapped[str]
    open_slot: Mapped[int | None]
    version: Mapped[int]
    created_at: Mapped[str]
    updated_at: Mapped[str]
    cursor: Mapped[int] = mapped_column(default=0, server_default="0")
    last_sequence: Mapped[int] = mapped_column(default=0, server_default="0")
    queue: Mapped[str] = mapped_column(Text, default="[]", server_default="[]")
    repairs: Mapped[str] = mapped_column(Text, default="[]", server_default="[]")
