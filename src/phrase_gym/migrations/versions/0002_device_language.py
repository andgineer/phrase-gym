"""Remove the obsolete server learning-language convenience preference."""

import json

import sqlalchemy as sa
from alembic import op

revision = "0002"
down_revision = "0001"
branch_labels = None
depends_on = None


def upgrade() -> None:
    connection = op.get_bind()
    settings = sa.table("app_settings", sa.column("id"), sa.column("payload"))
    for row in connection.execute(sa.select(settings)).mappings():
        payload = json.loads(row["payload"])
        payload.pop("last_learning_language", None)
        connection.execute(
            settings.update().where(settings.c.id == row["id"]).values(payload=json.dumps(payload)),
        )


def downgrade() -> None:
    # The old nullable field's default accepts the simplified payload.
    pass
