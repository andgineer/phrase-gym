"""Ordered evidence, FSRS state, retained takes and repair checkpoints."""

import json

import sqlalchemy as sa
from alembic import op

revision = "0004"
down_revision = "0003"
branch_labels = None
depends_on = None


def upgrade() -> None:
    for column in (
        sa.Column("cursor", sa.Integer, nullable=False, server_default="0"),
        sa.Column("last_sequence", sa.Integer, nullable=False, server_default="0"),
        sa.Column("queue", sa.Text, nullable=False, server_default="[]"),
        sa.Column("repairs", sa.Text, nullable=False, server_default="[]"),
    ):
        op.add_column("practice_session", column)
    with op.batch_alter_table("practice_session") as batch:
        batch.drop_constraint("session_slot_matches_status", type_="check")
        batch.create_check_constraint(
            "session_slot_matches_status",
            "(status IN ('active', 'paused') AND open_slot IS 1) OR "
            "(status IN ('superseded', 'completed') AND open_slot IS NULL)",
        )
    op.add_column("target_schedule", sa.Column("memory_state", sa.Text, nullable=True))
    op.add_column("target_schedule", sa.Column("policy_version", sa.String, nullable=True))
    op.create_table(
        "practice_event",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("session_id", sa.String, sa.ForeignKey("practice_session.id"), nullable=False),
        sa.Column("sequence", sa.Integer, nullable=False),
        sa.Column("payload", sa.Text, nullable=False),
        sa.Column("receipt", sa.Text, nullable=False),
        sa.UniqueConstraint("session_id", "sequence"),
    )
    op.create_table(
        "practice_attempt",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("session_id", sa.String, sa.ForeignKey("practice_session.id"), nullable=False),
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), nullable=False),
        sa.Column("step", sa.Integer, nullable=False),
        sa.Column("payload", sa.Text, nullable=False),
        sa.UniqueConstraint("session_id", "step"),
    )
    op.create_table(
        "review_receipt",
        sa.Column("group_id", sa.String, primary_key=True),
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), nullable=False),
        sa.Column("attempt_id", sa.String, sa.ForeignKey("practice_attempt.id"), nullable=False),
        sa.Column("payload", sa.Text, nullable=False),
    )
    op.create_table(
        "chunk_history",
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), primary_key=True),
        sa.Column("chunk_id", sa.String, primary_key=True),
        sa.Column("presentations", sa.Integer, nullable=False),
        sa.Column("recalls", sa.Integer, nullable=False),
        sa.Column("failures", sa.Integer, nullable=False),
        sa.Column("last_presented", sa.String, nullable=False),
    )
    op.create_table(
        "cue_report",
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), primary_key=True),
        sa.Column("revision", sa.Integer, primary_key=True),
        sa.Column("cue_id", sa.String, primary_key=True),
        sa.Column("reason", sa.Text, nullable=False),
    )
    op.create_table(
        "learner_recording",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), nullable=False),
        sa.Column("attempt_id", sa.String, sa.ForeignKey("practice_attempt.id"), nullable=True),
        sa.Column("exposure_id", sa.String, sa.ForeignKey("exposure.id"), nullable=True),
        sa.Column("metadata_json", sa.Text, nullable=False),
        sa.Column("audio", sa.LargeBinary, nullable=False),
        sa.Column("sha256", sa.String, nullable=False),
        sa.Column("received_at", sa.String, nullable=False),
    )
    connection = op.get_bind()
    sessions = sa.table(
        "practice_session",
        sa.column("id"),
        sa.column("manifest"),
        sa.column("queue"),
    )
    # Existing foundation sessions have no attempts, so their queue can be initialized safely.
    for row in connection.execute(sa.select(sessions)).mappings():
        manifest = json.loads(row["manifest"])
        queue = [
            {"target_index": i, "repair": False, "dialogue_turn": None}
            for i in range(len(manifest["targets"]))
        ]
        connection.execute(
            sessions.update()
            .where(sessions.c.id == row["id"])
            .values(
                queue=json.dumps(queue),
            ),
        )


def downgrade() -> None:
    for table in (
        "learner_recording",
        "cue_report",
        "chunk_history",
        "review_receipt",
        "practice_attempt",
        "practice_event",
    ):
        op.drop_table(table)
    with op.batch_alter_table("target_schedule") as batch:
        batch.drop_column("memory_state")
        batch.drop_column("policy_version")
    op.execute("UPDATE practice_session SET status = 'superseded' WHERE status = 'completed'")
    with op.batch_alter_table("practice_session") as batch:
        batch.drop_constraint("session_slot_matches_status", type_="check")
        batch.create_check_constraint(
            "session_slot_matches_status",
            "(status IN ('active', 'paused') AND open_slot IS 1) OR "
            "(status = 'superseded' AND open_slot IS NULL)",
        )
        for column in ("repairs", "queue", "last_sequence", "cursor"):
            batch.drop_column(column)
