import asyncio
from dataclasses import dataclass
from pathlib import Path


class OutputLimit(RuntimeError):
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
            raise OutputLimit("Child process exceeded its output limit")
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


async def run_bounded(
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
    except (TimeoutError, OutputLimit):
        await _stop(process)
        for task in tasks:
            task.cancel()
        await asyncio.gather(*tasks, return_exceptions=True)
        raise
