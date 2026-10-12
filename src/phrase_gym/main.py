"""Local tools share the publication boundary with HTTP requests."""

import base64
import json
from dataclasses import replace
from pathlib import Path
from uuid import UUID

import rich_click as click
import uvicorn
from pydantic import ValidationError

from phrase_gym import __version__
from phrase_gym.api.app import create_app
from phrase_gym.config import Config
from phrase_gym.content.imports import ImportService
from phrase_gym.content.validation import PackageValidator
from phrase_gym.contracts.content import ManualPackage
from phrase_gym.contracts.media import MediaMetadata, MediaUpload
from phrase_gym.errors import DomainError
from phrase_gym.media.service import MediaService
from phrase_gym.storage import Database

click.rich_click.USE_MARKDOWN = True


@click.group()
@click.version_option(version=__version__, prog_name="phrase-gym")
@click.option(
    "--data-dir",
    type=click.Path(path_type=Path),
    help="Private database/media directory.",
)
@click.pass_context
def phrase_gym(context: click.Context, data_dir: Path | None) -> None:
    """Run the local trainer foundation and import reviewed content."""
    runtime = Config.from_env()
    context.obj = replace(runtime, data_dir=data_dir) if data_dir else runtime


@phrase_gym.command()
@click.option("--port", default=8000, type=click.IntRange(1024, 65535), show_default=True)
@click.option("--client-dir", type=click.Path(exists=True, file_okay=False, path_type=Path))
@click.pass_obj
def serve(config: Config, port: int, client_dir: Path | None) -> None:
    """Serve the API and built client on loopback. HTTPS/private hosting comes later."""
    runtime = replace(config, client_dir=client_dir) if client_dir else config
    uvicorn.run(create_app(runtime), host="127.0.0.1", port=port, proxy_headers=False)


@phrase_gym.command("import-package")
@click.argument("package_path", type=click.Path(exists=True, dir_okay=False, path_type=Path))
@click.option(
    "--stage",
    is_flag=True,
    help="Persist the validated package for explicit publication.",
)
@click.pass_obj
def import_package(config: Config, package_path: Path, stage: bool) -> None:
    """Dry-run a manual JSON package by default; no automatic publication."""
    database = Database(config.data_dir)
    try:
        database.migrate()
        media = MediaService(database, config.data_dir)
        content = ImportService(database, PackageValidator(media.available))
        package = ManualPackage.model_validate_json(package_path.read_text(encoding="utf-8"))
        report = content.stage(package) if stage else content.validate(package)
        click.echo(report.model_dump_json(indent=2))
        if report.errors:
            raise click.ClickException("Package rejected; resolve the reported errors")
    except (ValidationError, DomainError) as error:
        raise click.ClickException(str(error)) from error
    finally:
        database.close()


@phrase_gym.command()
@click.argument("import_id")
@click.option("--reviewed", is_flag=True, required=True, help="Confirm linguistic/cue review.")
@click.pass_obj
def publish(config: Config, import_id: str, reviewed: bool) -> None:
    """Publish a staged package after reviewing its report and learning material."""

    if not reviewed:
        raise click.ClickException("Publication requires --reviewed")
    database = Database(config.data_dir)
    try:
        database.migrate()
        media = MediaService(database, config.data_dir)
        report = ImportService(database, PackageValidator(media.available)).publish(UUID(import_id))
        click.echo(report.model_dump_json(indent=2))
    except (ValueError, DomainError) as error:
        raise click.ClickException(str(error)) from error
    finally:
        database.close()


@phrase_gym.command("upload-reference")
@click.argument("audio_path", type=click.Path(exists=True, dir_okay=False, path_type=Path))
@click.argument("metadata_path", type=click.Path(exists=True, dir_okay=False, path_type=Path))
@click.option(
    "--mime-type",
    type=click.Choice(["audio/wav", "audio/mpeg", "audio/ogg", "audio/mp4"]),
    required=True,
)
@click.pass_obj
def upload_reference(
    config: Config,
    audio_path: Path,
    metadata_path: Path,
    mime_type: str,
) -> None:
    """Preserve an original clip and exact text/language/provenance; returns its asset ID."""
    database = Database(config.data_dir)
    try:
        database.migrate()
        metadata = MediaMetadata.model_validate_json(metadata_path.read_text(encoding="utf-8"))
        upload = MediaUpload.model_validate(
            {
                "metadata": metadata,
                "mime_type": mime_type,
                "audio_base64": base64.b64encode(audio_path.read_bytes()).decode(),
            },
        )
        click.echo(
            MediaService(database, config.data_dir).upload(upload).model_dump_json(indent=2),
        )
    except (ValidationError, DomainError) as error:
        raise click.ClickException(str(error)) from error
    finally:
        database.close()


@phrase_gym.command("reconcile-media")
@click.pass_obj
def reconcile_media(config: Config) -> None:
    """Report missing/orphan media; remove only old interrupted staging files."""
    database = Database(config.data_dir)
    try:
        database.migrate()
        click.echo(MediaService(database, config.data_dir).reconcile().model_dump_json(indent=2))
    finally:
        database.close()


@phrase_gym.command("export-schema")
@click.argument("output_path", type=click.Path(path_type=Path))
def export_schema(output_path: Path) -> None:
    """Export OpenAPI without starting the app or accessing learner data."""
    schema = create_app().openapi()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(schema, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":  # pragma: no cover
    phrase_gym()
