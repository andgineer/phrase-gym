"""Durable leased preparation and source-delivery receipts."""

import sqlalchemy as sa
from alembic import op

revision = "0005"
down_revision = "0004"
branch_labels = None
depends_on = None


def upgrade() -> None:
    op.create_table(
        "preparation_job",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("input_hash", sa.String, nullable=False, unique=True),
        sa.Column("payload", sa.Text, nullable=False),
        sa.Column("base_bindings", sa.Text, nullable=False),
        sa.Column("status", sa.String, nullable=False),
        sa.Column("attempts", sa.Integer, nullable=False),
        sa.Column("lease_token", sa.String),
        sa.Column("lease_until", sa.String),
        sa.Column("retry_at", sa.String, nullable=False),
        sa.Column("error", sa.String),
        sa.Column("result", sa.Text),
        sa.Column("review", sa.Text),
        sa.Column("published_import_id", sa.String),
    )
    op.create_table(
        "source_handoff",
        sa.Column("export_id", sa.String, primary_key=True),
        sa.Column("payload_hash", sa.String, nullable=False),
        sa.Column("payload", sa.Text, nullable=False),
        sa.Column("preparation_id", sa.String, nullable=False),
    )


def downgrade() -> None:
    op.drop_table("source_handoff")
    op.drop_table("preparation_job")
