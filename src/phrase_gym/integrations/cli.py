"""Read-only vault export and conversion of existing content into source intent."""

import json
from pathlib import Path

import rich_click as click
from sqlalchemy import select

from phrase_gym.config import Config
from phrase_gym.content.source_identity import source_changes
from phrase_gym.contracts.content import DialogueInput, LessonInput, TargetInput
from phrase_gym.contracts.preparation import SourcePackage, SourcePhrase
from phrase_gym.contracts.vault import VaultManifest
from phrase_gym.integrations.vault import export_vault
from phrase_gym.preparation.cli import runtime
from phrase_gym.tables import DialogueRow, LessonRow, Membership, SourceItem, Target, TargetRevision


@click.command("export-vault")
@click.argument("vault_root", type=click.Path(exists=True, file_okay=False, path_type=Path))
@click.argument("manifest_path", type=click.Path(exists=True, path_type=Path))
@click.option("--output", type=click.Path(path_type=Path))
@click.option("--upload-assets", is_flag=True)
@click.pass_obj
def vault_export(
    config: Config,
    vault_root: Path,
    manifest_path: Path,
    output: Path | None,
    upload_assets: bool,
):
    """Dry-run selected stable blocks by default; write export only outside the vault."""
    resolved_vault = vault_root.resolve()
    if within_vault(config.data_dir, resolved_vault):
        raise click.ClickException("Runtime data directory must stay outside the read-only vault")
    if output and within_vault(output, resolved_vault):
        raise click.ClickException("Export output must stay outside the read-only vault")
    with runtime(config) as (jobs, media):
        manifest = VaultManifest.model_validate_json(manifest_path.read_text())
        package, report = export_vault(vault_root, manifest, media if upload_assets else None)
        if package is not None:
            with jobs.db.read() as transaction:
                report.update(source_changes(transaction, package))
        click.echo(json.dumps(report, indent=2, ensure_ascii=False))
        if package is None or report.get("identity_errors"):
            raise click.ClickException("Vault export rejected")
        if output:
            if report["assets_not_uploaded"]:
                raise click.ClickException(
                    "Use --upload-assets before writing an importable audio-bound export",
                )
            output.write_text(package.model_dump_json(indent=2) + "\n")


@click.command("export-existing")
@click.argument("namespace")
@click.argument("native_language", type=click.Choice(["en", "ru", "sr", "de"]))
@click.argument("output", type=click.Path(path_type=Path))
@click.pass_obj
def existing_export(config: Config, namespace: str, native_language: str, output: Path):
    """Prepare/regenerate existing target content without touching progress or recordings."""
    with runtime(config) as (jobs, _media), jobs.db.read() as transaction:
        items = transaction.scalars(
            select(SourceItem).where(SourceItem.namespace == namespace),
        ).all()
        phrases = []
        language = None
        target_sources = {}
        for item in items:
            target = transaction.get(Target, item.target_id)
            assert target is not None
            language = target.language
            revision = transaction.get(TargetRevision, (target.id, target.current_revision))
            assert revision is not None
            content = TargetInput.model_validate_json(revision.payload)
            target_sources[target.id] = item.source_id
            phrases.append(
                SourcePhrase(
                    source_id=item.source_id,
                    kind=content.kind,
                    meaning_key=content.meaning_key,
                    recall_expression=content.recall_expression,
                    text=content.text,
                    purpose=content.purpose,
                    reference_audio_id=content.reference_audio_id,
                ),
            )
        lessons = []
        for lesson in transaction.scalars(
            select(LessonRow).where(LessonRow.namespace == namespace),
        ):
            members = transaction.scalars(
                select(Membership)
                .where(Membership.lesson_id == lesson.id)
                .order_by(Membership.position),
            ).all()
            lessons.append(
                LessonInput(
                    source_id=lesson.source_id,
                    title=lesson.title,
                    target_source_ids=[target_sources[m.target_id] for m in members],
                ),
            )
        if not phrases:
            raise click.ClickException("No source targets in this namespace")
        dialogues = latest_dialogues(transaction, namespace)
        package = SourcePackage.model_validate(
            {
                "namespace": namespace,
                "language": language,
                "native_language": native_language,
                "source_revision": "preparation-export-1",
                "phrases": phrases,
                "lessons": lessons,
                "dialogues": dialogues,
            },
        )
        output.write_text(package.model_dump_json(indent=2) + "\n")


def latest_dialogues(transaction, namespace):
    rows = transaction.scalars(
        select(DialogueRow)
        .where(DialogueRow.namespace == namespace)
        .order_by(DialogueRow.source_id, DialogueRow.revision.desc()),
    ).all()
    selected = {}
    for row in rows:
        if row.source_id not in selected:
            selected[row.source_id] = DialogueInput.model_validate_json(row.payload)
    return list(selected.values())


def within_vault(path: Path, root: Path) -> bool:
    resolved = path.resolve()
    if resolved.is_relative_to(root):
        return True
    # Case aliases can share an inode without Path.resolve normalizing their spelling.
    return any(
        parent.exists() and parent.samefile(root) for parent in (resolved, *resolved.parents)
    )
