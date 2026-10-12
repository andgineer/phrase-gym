from sqlalchemy.orm import Session

from phrase_gym.contracts.content import Cue, TargetInput
from phrase_gym.practice.tables import ChunkHistoryRow, CueReportRow


def select_cue(
    transaction: Session,
    target_id: str,
    revision: int,
    content: TargetInput,
) -> Cue | None:
    eligible = [
        cue
        for cue in content.cues
        if cue.reviewed
        and transaction.get(CueReportRow, (target_id, revision, cue.id)) is None
        and (
            content.kind == "fixed_phrase"
            or not set(content.required_chunks) & set(cue.visible_chunks)
        )
    ]
    if not eligible:
        return None
    baseline = len(set(content.required_chunks) - set(eligible[0].visible_chunks))
    comparable = [
        c for c in eligible if len(set(content.required_chunks) - set(c.visible_chunks)) == baseline
    ]

    def priority(cue: Cue) -> tuple[int, int, str, str]:
        histories = [
            transaction.get(ChunkHistoryRow, (target_id, chunk))
            for chunk in content.required_chunks
            if chunk not in cue.visible_chunks
        ]
        return (
            sum(h.recalls - h.failures for h in histories if h),
            sum(h.presentations for h in histories if h),
            max((h.last_presented for h in histories if h), default=""),
            cue.id,
        )

    return min(comparable, key=priority)
