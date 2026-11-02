from phrase_gym.server.contracts import CardKind, PracticeMode
from phrase_gym.server.import_contracts import StrictModel
from phrase_gym.server.practice_contracts import Rating


class PracticeDay(StrictModel):
    learning_date: str
    practiced: bool


class PracticeStreak(StrictModel):
    current: int
    best: int
    recent_days: list[PracticeDay]


class TodayCounts(StrictModel):
    completions: int
    first_meetings: int
    recalls: int
    skipped: int
    got_it: int
    effort: int
    not_yet: int
    recordings: int


class CumulativeWork(StrictModel):
    completions: int
    first_meetings: int
    practice_days: int


class WaitingCounts(StrictModel):
    new_count: int
    due_count: int


class ImportNotice(StrictModel):
    package_id: str
    source_label: str
    error: str


class PracticeFacts(StrictModel):
    learning_date: str
    streak: PracticeStreak
    today: TodayCounts
    work: CumulativeWork
    waiting: WaitingCounts
    notices: list[ImportNotice]


class StudiedPhrase(StrictModel):
    completion_id: str
    phrase_id: str | None
    text: str
    lesson_title: str
    mode: PracticeMode
    kind: CardKind
    rating: Rating | None
    skipped: bool


class StudiedPage(StrictModel):
    items: list[StudiedPhrase]
    next_offset: int | None
