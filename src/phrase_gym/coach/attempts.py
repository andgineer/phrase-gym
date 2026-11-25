import shutil
from collections.abc import Awaitable, Callable
from datetime import UTC, datetime
from pathlib import Path

from phrase_gym.coach.preparation import PreparationError
from phrase_gym.coach.usage import summarize


async def run_attempt[T](
    attempts: Path,
    usage: Path,
    request_id: str,
    prepare: Callable[[Path], Awaitable[T]],
    describe: Callable[[T], str],
) -> T:
    stamp = datetime.now(UTC).strftime("%Y%m%d-%H%M%S-%f")
    attempt = attempts / f"{stamp}-{request_id}"
    try:
        result = await prepare(attempt)
    except PreparationError as error:
        summarize(attempt, usage, request_id, f"failed: {error}")
        raise PreparationError(str(error), attempt, retry_helps=error.retry_helps) from error
    except (RuntimeError, OSError) as error:
        summarize(attempt, usage, request_id, f"failed: {error}")
        raise PreparationError(str(error), attempt) from error
    summarize(attempt, usage, request_id, describe(result))
    # Failed attempts stay for diagnosis; a published card needs no private trail.
    shutil.rmtree(attempt, ignore_errors=True)
    return result
