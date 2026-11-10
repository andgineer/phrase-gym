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


class PracticeWeek(StrictModel):
    practice_days: int
    # None while no target is chosen or the chosen one is paused.
    target: int | None


class PracticeTotals(StrictModel):
    completions: int
    first_meetings: int
    new_attempts: int
    # Rated attempts only; a Skip is counted in skipped.
    recalls: int
    skipped: int
    easy: int
    got_it: int
    effort: int
    not_yet: int
    recordings: int


class CumulativeWork(PracticeTotals):
    practice_days: int


class LibraryCounts(StrictModel):
    phrase_count: int
    due_count: int
    later_count: int
    new_count: int
    long_term_count: int


class ImportNotice(StrictModel):
    package_id: str
    source_label: str
    error: str


class PracticeFacts(StrictModel):
    learning_date: str
    streak: PracticeStreak
    week: PracticeWeek
    today: PracticeTotals
    work: CumulativeWork
    library: LibraryCounts
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
