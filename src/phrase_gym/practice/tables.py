from sqlalchemy import ForeignKey, LargeBinary, Text, UniqueConstraint
from sqlalchemy.orm import Mapped, mapped_column

from phrase_gym.tables import Base


class PracticeEvent(Base):
    __tablename__ = "practice_event"
    __table_args__ = (UniqueConstraint("session_id", "sequence"),)
    id: Mapped[str] = mapped_column(primary_key=True)
    session_id: Mapped[str] = mapped_column(ForeignKey("practice_session.id"))
    sequence: Mapped[int]
    payload: Mapped[str] = mapped_column(Text)
    receipt: Mapped[str] = mapped_column(Text)


class AttemptRow(Base):
    __tablename__ = "practice_attempt"
    __table_args__ = (UniqueConstraint("session_id", "step"),)
    id: Mapped[str] = mapped_column(primary_key=True)
    session_id: Mapped[str] = mapped_column(ForeignKey("practice_session.id"))
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"))
    step: Mapped[int]
    payload: Mapped[str] = mapped_column(Text)


class ReviewReceiptRow(Base):
    __tablename__ = "review_receipt"
    group_id: Mapped[str] = mapped_column(primary_key=True)
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"))
    attempt_id: Mapped[str] = mapped_column(ForeignKey("practice_attempt.id"))
    payload: Mapped[str] = mapped_column(Text)


class ChunkHistoryRow(Base):
    __tablename__ = "chunk_history"
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"), primary_key=True)
    chunk_id: Mapped[str] = mapped_column(primary_key=True)
    presentations: Mapped[int]
    recalls: Mapped[int]
    failures: Mapped[int]
    last_presented: Mapped[str]


class CueReportRow(Base):
    __tablename__ = "cue_report"
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"), primary_key=True)
    revision: Mapped[int] = mapped_column(primary_key=True)
    cue_id: Mapped[str] = mapped_column(primary_key=True)
    reason: Mapped[str] = mapped_column(Text)


class RecordingRow(Base):
    __tablename__ = "learner_recording"
    id: Mapped[str] = mapped_column(primary_key=True)
    target_id: Mapped[str] = mapped_column(ForeignKey("target.id"))
    attempt_id: Mapped[str | None] = mapped_column(ForeignKey("practice_attempt.id"))
    exposure_id: Mapped[str | None] = mapped_column(ForeignKey("exposure.id"))
    metadata_json: Mapped[str] = mapped_column(Text)
    audio: Mapped[bytes] = mapped_column(LargeBinary)
    sha256: Mapped[str]
    received_at: Mapped[str]
