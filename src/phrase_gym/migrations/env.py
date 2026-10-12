"""Migrations use the application's configured SQLite connection."""

from alembic import context

from phrase_gym.practice.tables import PracticeEvent

context.configure(
    connection=context.config.attributes["connection"],
    target_metadata=PracticeEvent.metadata,
)
with context.begin_transaction():
    context.run_migrations()
