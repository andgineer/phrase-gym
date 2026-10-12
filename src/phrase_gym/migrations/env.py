"""Migrations use the application's configured SQLite connection."""

from alembic import context

from phrase_gym.storage import Base

context.configure(
    connection=context.config.attributes["connection"],
    target_metadata=Base.metadata,
)
with context.begin_transaction():
    context.run_migrations()
