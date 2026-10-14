"""Read selected Markdown blocks and exact assets without writing source files."""

import base64
import hashlib
import re
from pathlib import Path

from phrase_gym.contracts.content import DialogueInput, DialogueTurn
from phrase_gym.contracts.media import MediaUpload
from phrase_gym.contracts.preparation import SourcePackage, SourcePhrase
from phrase_gym.contracts.vault import VaultManifest
from phrase_gym.errors import DomainError
from phrase_gym.integrations.legacy_notes import parse_note, selected_phrase
from phrase_gym.media.service import MediaService, validate_audio


def confined(root: Path, relative: Path) -> Path:
    path = (root / relative).resolve()
    if relative.is_absolute() or not path.is_relative_to(root.resolve()):
        raise DomainError("Vault paths must stay within the explicitly selected root", 422)
    return path


def extract_block(note: str, block_id: str) -> str:
    marker = re.compile(r"\s+\^" + re.escape(block_id) + r"\s*$")
    matches = [marker.sub("", line).strip() for line in note.splitlines() if marker.search(line)]
    if len(matches) != 1:
        raise DomainError("Selected block ID is missing or ambiguous", 422)
    text = re.sub(r"^\s*(?:[-*+] |\d+\. )", "", matches[0])
    if not text:
        raise DomainError("Selected phrase block is empty", 422)
    return text


def export_vault(
    root: Path,
    manifest: VaultManifest,
    media: MediaService | None = None,
) -> tuple[SourcePackage | None, dict]:
    phrases = []
    parsed_notes = {}
    errors = []
    digest = hashlib.sha256()
    for selection in manifest.selections:
        try:
            raw, text, context = read_selection(root, selection, parsed_notes)
            digest.update(selection.source_id.encode() + raw)
            audio_id = None
            if selection.attachment is not None:
                audio = confined(root, selection.attachment).read_bytes()
                metadata = selection.audio_metadata
                if (
                    metadata is None
                    or metadata.text != text
                    or metadata.language != manifest.language
                ):
                    raise DomainError("Attachment requires exact phrase/language provenance", 422)
                upload = MediaUpload.model_validate(
                    {
                        "metadata": metadata,
                        "mime_type": selection.mime_type,
                        "audio_base64": base64.b64encode(audio).decode(),
                    },
                )
                validate_audio(audio, upload.mime_type)
                digest.update(audio)
                if media:
                    audio_id = media.upload(upload).id
            phrases.append(
                SourcePhrase(
                    source_id=selection.source_id,
                    kind=selection.kind,
                    meaning_key=selection.meaning_key,
                    recall_expression=selection.recall_expression,
                    text=text,
                    purpose=selection.purpose,
                    context=context,
                    reference_audio_id=audio_id,
                ),
            )
        except (OSError, UnicodeError, DomainError, ValueError) as error:
            errors.append({"source_id": selection.source_id, "error": str(error)})
    ids = [p.source_id for p in phrases]
    if len(ids) != len(set(ids)):
        errors.append({"source_id": "manifest", "error": "Duplicate stable source ID"})
    report = {
        "selected": len(manifest.selections),
        "valid": len(phrases),
        "errors": errors,
        "missing_audio": [s.source_id for s in manifest.selections if s.attachment is None],
        "assets_not_uploaded": [
            s.source_id for s in manifest.selections if s.attachment is not None and media is None
        ],
        "duplicate_surface_forms": sorted(
            {p.text for p in phrases if sum(q.text == p.text for q in phrases) > 1},
        ),
    }
    if errors:
        return None, report
    dialogues = manifest.dialogues
    if manifest.include_dialogues:
        try:
            dialogues = dialogues + export_dialogues(root, manifest, parsed_notes, phrases, media)
        except (OSError, UnicodeError, DomainError, ValueError) as error:
            errors.append({"source_id": "dialogues", "error": str(error)})
            return None, report
    package = SourcePackage(
        namespace=manifest.namespace,
        language=manifest.language,
        native_language=manifest.native_language,
        source_revision=digest.hexdigest(),
        phrases=phrases,
        lessons=manifest.lessons,
        dialogues=dialogues,
    )
    return package, report


def export_dialogues(root, manifest, parsed_notes, phrases, media):
    dialogues = []
    phrase_by_id = {p.source_id: p for p in phrases}
    for parsed in parsed_notes.values():
        for position, dialogue in enumerate(parsed.dialogues):
            turns = []
            for speaker, text, attachment in dialogue.turns:
                matches = [
                    selection.source_id
                    for selection in manifest.selections
                    if attachment is not None
                    and selection.source_audio_name == attachment
                    and phrase_by_id[selection.source_id].text == text
                ]
                if len(matches) > 1:
                    raise DomainError(
                        "Ambiguous dialogue target mapping: "
                        + ", ".join(sorted(matches))
                        + "; disable automatic dialogue export and supply explicit turn bindings",
                        422,
                    )
                linked = matches[0] if matches else None
                audio_id = phrase_by_id[linked].reference_audio_id if linked else None
                if attachment in manifest.assets:
                    asset = manifest.assets[attachment]
                    if asset.metadata.text != text or asset.metadata.language != manifest.language:
                        raise DomainError(
                            "Dialogue audio requires exact turn/language provenance",
                            422,
                        )
                    if media:
                        audio_id = media.upload(
                            MediaUpload.model_validate(
                                {
                                    "metadata": asset.metadata,
                                    "mime_type": asset.mime_type,
                                    "audio_base64": base64.b64encode(
                                        confined(root, asset.path).read_bytes(),
                                    ).decode(),
                                },
                            ),
                        ).id
                turns.append(
                    DialogueTurn(
                        speaker=speaker,
                        text=text,
                        target_source_id=linked,
                        audio_id=audio_id,
                    ),
                )
            dialogues.append(
                DialogueInput(
                    source_id=f"{parsed.lesson_id}.dialogue.{position}",
                    title=dialogue.title,
                    turns=turns,
                ),
            )
    return dialogues


def read_selection(root, selection, parsed_notes):
    path = confined(root, selection.note)
    raw = path.read_bytes()
    note_text = raw.decode("utf-8")
    context = selection.context
    if selection.block_id:
        text = extract_block(note_text, selection.block_id)
    elif selection.source_audio_name:
        parsed_note = parse_note(note_text)
        parsed_notes[str(selection.note)] = parsed_note
        parsed = selected_phrase(parsed_note, selection.source_audio_name)
        text = parsed.text
        context = context or parsed.context
    else:
        raise DomainError("Select a stable block ID or legacy source audio name", 422)
    return raw, text, context
