"""The existing selected vault lesson syntax, without its historical hint rules."""

import re
from dataclasses import dataclass, field

from phrase_gym.errors import DomainError

FRONTMATTER_PARTS = 3

EMBED = re.compile(r"!\[\[([^\]]+)\]\]")
NUMBERED = re.compile(r"^\s*\d+\.\s+(.+)")
TURN = re.compile(r"^- ([^:]+):\s*(.+)")


@dataclass
class NotePhrase:
    text: str
    attachment: str | None = None
    context: str | None = None


@dataclass
class NoteDialogue:
    title: str
    turns: list[tuple[str, str, str | None]] = field(default_factory=list)


@dataclass
class ParsedNote:
    lesson_id: str
    phrases: list[NotePhrase]
    dialogues: list[NoteDialogue]


def parse_note(note: str) -> ParsedNote:
    frontmatter = note.split("---", 2)
    if len(frontmatter) < FRONTMATTER_PARTS or frontmatter[0].strip():
        raise DomainError("Legacy lesson requires front matter with a stable id", 422)
    match = re.search(r"^id:\s*[\"']?([^\n\"']+)", frontmatter[1], re.MULTILINE)
    if not match:
        raise DomainError("Legacy lesson front matter has no id", 422)
    parsed = ParsedNote(match.group(1).strip(), [], [])
    section = None
    for raw in frontmatter[2].splitlines():
        line = raw.strip()
        if line.startswith("## "):
            section = line[3:]
            continue
        if section == "фразы":
            parse_phrase_line(parsed, raw, line)
        elif section == "диалоги":
            parse_dialogue_line(parsed, line)
    return parsed


def parse_phrase_line(parsed: ParsedNote, raw: str, line: str) -> None:
    numbered = NUMBERED.match(raw)
    if numbered:
        parsed.phrases.append(NotePhrase(numbered.group(1)))
    elif parsed.phrases and (embed := EMBED.search(line)):
        parsed.phrases[-1].attachment = embed.group(1)
    elif parsed.phrases and line.startswith("*") and line.endswith("*"):
        parsed.phrases[-1].context = line.strip("*")


def parse_dialogue_line(parsed: ParsedNote, line: str) -> None:
    if line.startswith("### "):
        parsed.dialogues.append(NoteDialogue(line[4:]))
    elif parsed.dialogues and (turn := TURN.match(line)):
        text = re.sub(r"<!--.*?-->", "", turn.group(2))
        embed = EMBED.search(text)
        text = EMBED.sub("", text).replace("**", "").strip()
        parsed.dialogues[-1].turns.append((turn.group(1), text, embed.group(1) if embed else None))


def selected_phrase(parsed: ParsedNote, attachment: str) -> NotePhrase:
    matches = [p for p in parsed.phrases if p.attachment == attachment]
    if len(matches) != 1:
        raise DomainError("Selected attachment mapping is missing or ambiguous", 422)
    return matches[0]
