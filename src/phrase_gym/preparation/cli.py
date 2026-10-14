"""Operator preparation tools; no background provider calls from serving practice."""

import asyncio
import json
from contextlib import contextmanager
from pathlib import Path
from uuid import UUID

import rich_click as click
from pydantic import ValidationError

from phrase_gym.config import Config
from phrase_gym.content.imports import ImportService
from phrase_gym.content.validation import PackageValidator
from phrase_gym.contracts.handoff import HandoffEnvelope
from phrase_gym.contracts.preparation import HumanReview, PreparationDraft, SourcePackage
from phrase_gym.errors import DomainError
from phrase_gym.integrations.handoff import HandoffReceiver
from phrase_gym.media.service import MediaService
from phrase_gym.preparation.config import PreparationSettings
from phrase_gym.preparation.jobs import PreparationJobs, digest
from phrase_gym.preparation.providers import BrokerProvider
from phrase_gym.preparation.publication import publish_reviewed
from phrase_gym.preparation.revisions import revise_draft
from phrase_gym.preparation.speech import SpeechAdapter
from phrase_gym.preparation.worker import PreparationWorker
from phrase_gym.storage import Database


@contextmanager
def runtime(config: Config):
    database = Database(config.data_dir)
    try:
        database.migrate()
        media = MediaService(database, config.data_dir)
        yield PreparationJobs(database), media
    except (DomainError, ValidationError, ValueError, OSError) as error:
        raise click.ClickException(str(error)) from error
    finally:
        database.close()


def settings(path: Path) -> PreparationSettings:
    return PreparationSettings.model_validate_json(path.read_text())


@click.group("prepare")
def preparation_cli():
    """Queue, inspect, retry and explicitly review preparation drafts."""


@preparation_cli.command("enqueue")
@click.argument("source_path", type=click.Path(exists=True, path_type=Path))
@click.argument("config_path", type=click.Path(exists=True, path_type=Path))
@click.pass_obj
def enqueue(config: Config, source_path: Path, config_path: Path):
    with runtime(config) as (jobs, _media):
        source = SourcePackage.model_validate_json(source_path.read_text())
        click.echo(jobs.enqueue(source, settings(config_path).identity()).model_dump_json(indent=2))


@preparation_cli.command("inspect")
@click.argument("job_id", type=UUID)
@click.pass_obj
def inspect_job(config: Config, job_id: UUID):
    with runtime(config) as (jobs, _media):
        details = jobs.details(job_id)
        if details["result"]:
            details["draft_hash"] = digest(json.dumps(details["result"]))
        click.echo(json.dumps(details, ensure_ascii=False, indent=2))


@preparation_cli.command("retry")
@click.argument("job_id", type=UUID)
@click.option(
    "--regenerate",
    is_flag=True,
    help="Retain failed answers in history and request new candidates.",
)
@click.pass_obj
def retry(config: Config, job_id: UUID, regenerate: bool):
    with runtime(config) as (jobs, _media):
        click.echo(jobs.retry(job_id, regenerate=regenerate).model_dump_json(indent=2))


@preparation_cli.command("work")
@click.argument("config_path", type=click.Path(exists=True, path_type=Path))
@click.option("--limit", type=click.IntRange(1, 100), default=1)
@click.pass_obj
def work(config: Config, config_path: Path, limit: int):
    """Run a bounded number of leased jobs with this exact operator configuration."""
    with runtime(config) as (jobs, media):
        configured = settings(config_path)
        asyncio.run(run_worker(jobs, media, config, configured, limit))


async def run_worker(jobs, media, config, configured, limit):
    provider = BrokerProvider(configured.provider, config.data_dir / "llmbroker")
    try:
        worker = PreparationWorker(
            jobs,
            provider,
            SpeechAdapter(media, configured.speech),
            configured.identity(),
        )
        for _ in range(limit):
            if not await worker.once():
                break
    finally:
        await provider.close()


@preparation_cli.command("providers")
@click.argument("config_path", type=click.Path(exists=True, path_type=Path))
@click.pass_obj
def providers(config: Config, config_path: Path):
    """Inspect key availability/model state without printing secret material."""

    async def inspect():
        provider = BrokerProvider(settings(config_path).provider, config.data_dir / "llmbroker")
        try:
            click.echo(json.dumps(await provider.availability(), indent=2))
        finally:
            await provider.close()

    asyncio.run(inspect())


@preparation_cli.command("publish")
@click.argument("job_id", type=UUID)
@click.argument("review_path", type=click.Path(exists=True, path_type=Path))
@click.pass_obj
def publish(config: Config, job_id: UUID, review_path: Path):
    """Apply a hash-bound human review, including actual reference listening approval."""
    with runtime(config) as (jobs, media):
        review = HumanReview.model_validate_json(review_path.read_text())
        imports = ImportService(jobs.db, PackageValidator(media.available))
        click.echo(publish_reviewed(jobs, imports, job_id, review).model_dump_json(indent=2))


@preparation_cli.command("receive")
@click.argument("envelope_path", type=click.Path(exists=True, path_type=Path))
@click.argument("config_path", type=click.Path(exists=True, path_type=Path))
@click.pass_obj
def receive(config: Config, envelope_path: Path, config_path: Path):
    with runtime(config) as (jobs, _media):
        envelope = HandoffEnvelope.model_validate_json(envelope_path.read_text())
        receiver = HandoffReceiver(jobs, settings(config_path).identity())
        click.echo(receiver.receive(envelope).model_dump_json(indent=2))


@preparation_cli.command("revise")
@click.argument("job_id", type=UUID)
@click.argument("draft_path", type=click.Path(exists=True, path_type=Path))
@click.option("--reviewer", required=True)
@click.option("--notes", required=True)
@click.pass_obj
def revise(config: Config, job_id: UUID, draft_path: Path, reviewer: str, notes: str):
    """Correct a withheld draft; preserve raw answers and require a new approval hash."""
    with runtime(config) as (jobs, _media):
        draft = PreparationDraft.model_validate_json(draft_path.read_text())
        click.echo(revise_draft(jobs, job_id, draft, reviewer, notes).model_dump_json(indent=2))
