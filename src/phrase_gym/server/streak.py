from collections.abc import Iterable
from datetime import date, timedelta

from phrase_gym.server.facts_contracts import PracticeDay, PracticeStreak

RECENT_DAYS = 7
ONE_DAY = timedelta(days=1)


def _run_ending(days: set[date], last: date) -> int:
    length = 0
    while last - length * ONE_DAY in days:
        length += 1
    return length


def practice_streak(learning_dates: Iterable[str], today: str) -> PracticeStreak:
    days = {date.fromisoformat(value) for value in learning_dates}
    current_day = date.fromisoformat(today)
    # Until today's first completion, the run that ended yesterday is still current.
    anchor = current_day if current_day in days else current_day - ONE_DAY
    recent = [current_day - offset * ONE_DAY for offset in reversed(range(RECENT_DAYS))]
    return PracticeStreak(
        current=_run_ending(days, anchor),
        best=max((_run_ending(days, day) for day in days if day + ONE_DAY not in days), default=0),
        recent_days=[
            PracticeDay(learning_date=day.isoformat(), practiced=day in days) for day in recent
        ],
    )
