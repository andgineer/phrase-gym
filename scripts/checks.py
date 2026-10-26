import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MAX_MODULE_LINES = 300


def run(*command: str) -> None:
    subprocess.run(command, cwd=ROOT, check=True)


def module_sizes() -> None:
    oversized = []
    for pattern in ("src/phrase_gym/*.py", "frontend/src/*.ts", "frontend/src/*.vue"):
        for path in ROOT.glob(pattern):
            if path.name.endswith(".generated.ts"):
                continue
            lines = len(path.read_text().splitlines())
            if lines > MAX_MODULE_LINES:
                oversized.append(f"{path.relative_to(ROOT)}: {lines} lines")
    if oversized:
        raise SystemExit(
            "Handwritten production modules exceed 300 lines:\n" + "\n".join(oversized),
        )


def pre() -> None:
    run("uv", "run", "pre-commit", "run", "--all-files")


def project_pre() -> None:
    run("uv", "lock", "--check")
    run("uv", "run", "mypy", "src")
    module_sizes()
    run("uv", "run", "python", "scripts/generate_api.py")
    run("npm", "--prefix", "frontend", "run", "api-types")
    run(
        "git",
        "diff",
        "--exit-code",
        "--",
        "frontend/openapi.json",
        "frontend/src/api.generated.ts",
    )
    run("npm", "--prefix", "frontend", "run", "format:check")


def test() -> None:
    run("uv", "run", "pytest", "--cov=phrase_gym", "--cov-report=term-missing")
    run("npm", "--prefix", "frontend", "run", "typecheck")
    run("npm", "--prefix", "frontend", "test")
    run("npm", "--prefix", "frontend", "run", "build")
    run("npm", "--prefix", "frontend", "run", "test:browser")


if __name__ == "__main__":
    commands = {"pre": pre, "project-pre": project_pre, "test": test}
    if len(sys.argv[1:]) != 1 or sys.argv[1] not in commands:
        raise SystemExit("Usage: python scripts/checks.py [pre|project-pre|test]")
    commands[sys.argv[1]]()
