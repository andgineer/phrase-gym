"""Pinned sessions and one shared due/version record per target."""

import sqlalchemy as sa
from alembic import op

revision = "0003"
down_revision = "0002"
branch_labels = None
depends_on = None


def upgrade() -> None:
    op.create_table(
        "target_schedule",
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), primary_key=True),
        sa.Column("version", sa.Integer, nullable=False),
        sa.Column("due_at", sa.String, nullable=False),
    )
    op.create_table(
        "practice_session",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("start_payload", sa.Text, nullable=False),
        sa.Column("manifest", sa.Text, nullable=False),
        sa.Column("status", sa.String, nullable=False),
        sa.Column("open_slot", sa.Integer, nullable=True),
        sa.Column("version", sa.Integer, nullable=False),
        sa.Column("created_at", sa.String, nullable=False),
        sa.Column("updated_at", sa.String, nullable=False),
        sa.UniqueConstraint("open_slot", name="one_open_practice_session"),
        sa.CheckConstraint(
            "(status IN ('active', 'paused') AND open_slot IS 1) OR "
            "(status = 'superseded' AND open_slot IS NULL)",
            name="session_slot_matches_status",
        ),
    )


def downgrade() -> None:
    op.drop_table("practice_session")
    op.drop_table("target_schedule")
