"""Initial content, settings, imports, reference media and exposure schema."""

import sqlalchemy as sa
from alembic import op

revision = "0001"
down_revision = None
branch_labels = None
depends_on = None


def upgrade() -> None:
    op.create_table(
        "app_settings",
        sa.Column("id", sa.Integer, primary_key=True),
        sa.Column("payload", sa.Text, nullable=False),
    )
    op.create_table(
        "target",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("language", sa.String, nullable=False),
        sa.Column("kind", sa.String, nullable=False),
        sa.Column("meaning_key", sa.String, nullable=False),
        sa.Column("recall_expression", sa.Text, nullable=False),
        sa.Column("current_revision", sa.Integer, nullable=False),
    )
    op.create_table(
        "target_revision",
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), primary_key=True),
        sa.Column("revision", sa.Integer, primary_key=True),
        sa.Column("payload", sa.Text, nullable=False),
        sa.Column("fingerprint", sa.String, nullable=False),
        sa.Column("created_at", sa.String, nullable=False),
    )
    op.create_table(
        "source_item",
        sa.Column("namespace", sa.String, primary_key=True),
        sa.Column("source_id", sa.String, primary_key=True),
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), nullable=False),
        sa.Column("source_revision", sa.String, nullable=False),
    )
    op.create_table(
        "lesson",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("namespace", sa.String, nullable=False),
        sa.Column("source_id", sa.String, nullable=False),
        sa.Column("language", sa.String, nullable=False),
        sa.Column("title", sa.Text, nullable=False),
        sa.UniqueConstraint("namespace", "source_id"),
    )
    op.create_table(
        "lesson_membership",
        sa.Column("lesson_id", sa.String, sa.ForeignKey("lesson.id"), primary_key=True),
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), primary_key=True),
        sa.Column("position", sa.Integer, nullable=False),
    )
    op.create_table(
        "dialogue",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("revision", sa.Integer, primary_key=True),
        sa.Column("namespace", sa.String, nullable=False),
        sa.Column("source_id", sa.String, nullable=False),
        sa.Column("language", sa.String, nullable=False),
        sa.Column("payload", sa.Text, nullable=False),
        sa.Column("target_ids", sa.Text, nullable=False),
        sa.UniqueConstraint("namespace", "source_id", "revision"),
    )
    op.create_table(
        "content_import",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("payload", sa.Text, nullable=False),
        sa.Column("report", sa.Text, nullable=False),
        sa.Column("status", sa.String, nullable=False),
        sa.Column("created_at", sa.String, nullable=False),
    )
    op.create_table(
        "media",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("sha256", sa.String, nullable=False),
        sa.Column("byte_length", sa.Integer, nullable=False),
        sa.Column("mime_type", sa.String, nullable=False),
        sa.Column("metadata_json", sa.Text, nullable=False),
    )
    op.create_table(
        "exposure",
        sa.Column("id", sa.String, primary_key=True),
        sa.Column("target_id", sa.String, sa.ForeignKey("target.id"), nullable=False),
        sa.Column("revision", sa.Integer, nullable=False),
        sa.Column("payload", sa.Text, nullable=False),
        sa.Column("received_at", sa.String, nullable=False),
    )


def downgrade() -> None:
    for table in (
        "exposure",
        "media",
        "content_import",
        "dialogue",
        "lesson_membership",
        "lesson",
        "source_item",
        "target_revision",
        "target",
        "app_settings",
    ):
        op.drop_table(table)
