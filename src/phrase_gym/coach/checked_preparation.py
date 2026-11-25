import json
from dataclasses import dataclass
from pathlib import Path

from phrase_gym.coach.attempts import run_attempt
from phrase_gym.coach.basic_card import confirmed_card
from phrase_gym.coach.drafts import Check, Draft, parse_check, parse_draft
from phrase_gym.coach.enrichment import enriched_phrase
from phrase_gym.coach.messages import message
from phrase_gym.coach.preparation import Cli, PreparationError, learner_input, render, schema
from phrase_gym.server.coach_contracts import BasicCard, CoachCorrection, CoachWork
from phrase_gym.server.import_contracts import ImportPhrase

# Both judgments must establish the same decision, studied form and any correction.
AGREED = ("outcome", "original_span", "target")


@dataclass(frozen=True)
class PreparedPhrase:
    phrase: ImportPhrase
    correction: CoachCorrection | None


def checked_card(draft: Draft, check: Check, work: CoachWork) -> BasicCard | PreparedPhrase:
    proposed, confirmed = draft.recognition, check.recognition
    if any(getattr(proposed, key) != getattr(confirmed, key) for key in AGREED):
        # A disputed rejection or form is a judgment, so sending the same text again cannot help.
        raise PreparationError(message("unresolved", work.help_language), retry_helps=False)
    if confirmed.outcome == "reject":
        raise PreparationError(confirmed.reason.strip(), retry_helps=False)
    usable = proposed.source_usable and confirmed.source_usable
    source = confirmed.source if usable and proposed.source == confirmed.source else None
    target, original = confirmed.target, confirmed.original_span
    if proposed.kind == confirmed.kind:
        phrase = enriched_phrase(draft, check, confirmed, source, work.package_id)
        if phrase is not None:
            changed = original != target
            correction = CoachCorrection(original=original, corrected=target) if changed else None
            return PreparedPhrase(phrase, correction)
    return confirmed_card(target, original, source, work.card_language, work.help_language)


def _kind(card: BasicCard | PreparedPhrase) -> str:
    return "basic" if isinstance(card, BasicCard) else "enriched"


class CheckedPreparer:
    """A writer drafts the card; a separate checker judges the input and every addition."""

    def __init__(self, writer: Cli, checker: Cli, attempts: Path):
        self.writer = writer
        self.checker = checker
        self.attempts = attempts

    async def prepare(self, work: CoachWork) -> BasicCard | PreparedPhrase:
        async def write_and_check(attempt: Path) -> BasicCard | PreparedPhrase:
            typed = learner_input(work)
            written = await self.writer.ask(
                attempt,
                "write",
                render("write.md", input=typed),
                schema("write.schema.json"),
            )
            draft = parse_draft(written, work.text)
            shown = json.dumps(written, ensure_ascii=False, indent=2)
            checked = await self.checker.ask(
                attempt,
                "check",
                render("check.md", input=typed, draft=shown),
                schema("check.schema.json"),
            )
            return checked_card(draft, parse_check(checked, draft, work.text), work)

        return await run_attempt(self.attempts, work.package_id, write_and_check, _kind)
