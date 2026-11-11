import asyncio
from dataclasses import dataclass
from pathlib import Path


class OutputLimitError(RuntimeError):
    pass


@dataclass(frozen=True)
class ProcessResult:
    returncode: int
    stdout: bytes
    stderr: bytes


async def _read_limited(reader: asyncio.StreamReader, limit: int) -> bytes:
    output = bytearray()
    while chunk := await reader.read(65_536):
        output.extend(chunk)
        if len(output) > limit:
            raise OutputLimitError("Child process exceeded its output limit")
    return bytes(output)


async def _stop(process: asyncio.subprocess.Process) -> None:
    if process.returncode is not None:
        return
    process.terminate()
    try:
        await asyncio.wait_for(process.wait(), 1)
    except TimeoutError:
        process.kill()
        await process.wait()


async def _cleanup(
    process: asyncio.subprocess.Process,
    tasks: tuple[asyncio.Task[bytes], asyncio.Task[bytes], asyncio.Task[int]],
) -> None:
    for task in tasks:
        task.cancel()
    await _stop(process)
    await asyncio.gather(*tasks, return_exceptions=True)


# Keep each resource limit explicit at the subprocess boundary.
async def run_bounded(  # noqa: PLR0913
    program: Path,
    arguments: list[str],
    *,
    cwd: Path,
    environment: dict[str, str],
    timeout: float,
    stdout_limit: int,
    stderr_limit: int,
) -> ProcessResult:
    process = await asyncio.create_subprocess_exec(
        str(program),
        *arguments,
        cwd=cwd,
        env=environment,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    assert process.stdout is not None and process.stderr is not None
    stdout_task = asyncio.create_task(_read_limited(process.stdout, stdout_limit))
    stderr_task = asyncio.create_task(_read_limited(process.stderr, stderr_limit))
    wait_task = asyncio.create_task(process.wait())
    tasks = (stdout_task, stderr_task, wait_task)
    try:
        async with asyncio.timeout(timeout):
            stdout, stderr, returncode = await asyncio.gather(stdout_task, stderr_task, wait_task)
        return ProcessResult(returncode, stdout, stderr)
    except BaseException:
        cleanup = asyncio.create_task(_cleanup(process, tasks))
        try:
            await asyncio.shield(cleanup)
        except asyncio.CancelledError:
            await cleanup
        raise


async def _watch_sizes(paths: tuple[Path, ...], limit: int) -> None:
    while True:
        if sum(path.stat().st_size for path in paths if path.exists()) > limit:
            raise OutputLimitError("Child process exceeded its output limit")
        await asyncio.sleep(0.5)


# Output goes straight to files so a timed-out or oversized run still leaves its logs.
async def run_to_files(  # noqa: PLR0913
    program: Path,
    arguments: list[str],
    *,
    cwd: Path,
    environment: dict[str, str],
    timeout: float,
    output: tuple[Path, Path],
    limit: int,
) -> int:
    stdout_path, stderr_path = output
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = await asyncio.create_subprocess_exec(
            str(program),
            *arguments,
            cwd=cwd,
            env=environment,
            stdin=asyncio.subprocess.DEVNULL,
            stdout=stdout,
            stderr=stderr,
        )
        watcher = asyncio.create_task(_watch_sizes(output, limit))
        waiter = asyncio.create_task(process.wait())
        try:
            async with asyncio.timeout(timeout):
                done, _ = await asyncio.wait(
                    {watcher, waiter},
                    return_when=asyncio.FIRST_COMPLETED,
                )
            if watcher in done:
                watcher.result()
            return waiter.result()
        finally:
            watcher.cancel()
            await asyncio.shield(_stop(process))
            await asyncio.gather(watcher, waiter, return_exceptions=True)
