import json
import shlex
import shutil
import sys
from contextlib import contextmanager
from pathlib import Path

from invoke import Collection, Context, task

from phrase_gym.__about__ import __version__
from phrase_gym.server.boundary import tailscale_host

DOCS_PATH = Path("docs")
DOCS_SRC_PATH = DOCS_PATH / "src"
ALLOWED_DOC_LANGUAGES = ["en"] + sorted(
    path.name for path in DOCS_SRC_PATH.iterdir() if path.is_dir() and path.name != "en"
)
ALLOWED_VERSION_TYPES = ["release", "bug", "feature"]
MAX_MODULE_LINES = 300


@task
def version(_context: Context):
    """Show the package version."""
    print(__version__)


def ver_task_factory(version_type: str):
    @task
    def ver(context: Context, dry_run=False):
        """Bump, commit and push a version tag; --dry-run only previews it."""
        suffix = " --dry-run" if dry_run else ""
        context.run(f"./scripts/verup.sh {version_type}{suffix}")

    return ver


@task
def reqs(context: Context):
    """Upgrade the dependency lock and pre-commit hooks."""
    context.run("uv run pre-commit autoupdate")
    context.run("uv lock --upgrade")


@contextmanager
def docs_rendered(language: str):
    config = (DOCS_PATH / "mkdocs.yml").read_text()
    build_path = Path("build/docs")
    config_path = build_path / "mkdocs.yml"
    source_path = build_path / "src" / language
    site_dir = "site" if language == "en" else f"site/{language}"
    build_path.mkdir(parents=True, exist_ok=True)
    config_path.write_text(config.replace("LANGUAGE", language).replace("SITE_DIR", site_dir))
    shutil.rmtree(source_path, ignore_errors=True)
    shutil.copytree(DOCS_SRC_PATH / language, source_path)
    yield config_path


def docs_task_factory(language: str):
    @task
    def docs(context: Context):
        """Preview documentation at http://127.0.0.1:8001."""
        with docs_rendered(language) as config:
            context.run(f"uv run zensical serve -f {shlex.quote(str(config))} -a 127.0.0.1:8001")

    return docs


@task
def build_docs(context: Context):
    """Build the documentation site in build/docs/site/."""
    for language in ALLOWED_DOC_LANGUAGES:
        with docs_rendered(language) as config:
            context.run(f"uv run zensical build --strict -f {shlex.quote(str(config))}")


@task
def uv(context: Context):
    """Install or upgrade uv using its official installer."""
    context.run("curl -LsSf https://astral.sh/uv/install.sh | sh")


@task
def pre(context: Context):
    """Run all pre-commit checks."""
    context.run("uv run pre-commit run --verbose --all-files")


@task
def check_sizes(_context: Context):
    """Check handwritten production module sizes."""
    oversized = []
    for root in (Path("src/phrase_gym"), Path("frontend/src")):
        for path in root.rglob("*"):
            if path.suffix not in {".py", ".ts", ".vue"} or "static" in path.parts:
                continue
            if path.name.endswith(".generated.ts"):
                continue
            lines = len(path.read_text().splitlines())
            if lines > MAX_MODULE_LINES:
                oversized.append(f"{path}: {lines} lines")
    if oversized:
        raise SystemExit(
            "Handwritten production modules exceed 300 lines:\n" + "\n".join(oversized),
        )


@task
def check_api(context: Context):
    """Regenerate the API contract and reject uncommitted drift."""
    context.run("uv run python scripts/generate_api.py")
    context.run("npm --prefix frontend run api-types")
    context.run("git diff --exit-code -- frontend/openapi.json frontend/src/api.generated.ts")


@task
def build_frontend(context: Context):
    """Build the Vue client into the Python package."""
    context.run("npm --prefix frontend run build")


@task
def build(context: Context):
    """Build the frontend, wheel and source distribution."""
    context.run("npm --prefix frontend ci")
    build_frontend(context)
    context.run("uv build")


@task
def run(context: Context, build=False, tailscale=False):
    """Run the server; optionally build the client and connect private Tailscale HTTPS."""
    if build:
        build_frontend(context)
    command = "uv run phrase-gym run"
    if tailscale:
        status = context.run("tailscale status --json", hide=True, pty=False)
        hostname = json.loads(status.stdout)["Self"]["DNSName"].rstrip(".")
        origin = f"https://{hostname}"
        tailscale_host(origin)
        context.run("tailscale serve --bg --yes http://127.0.0.1:8000")
        command += f" --tailscale-origin {shlex.quote(origin)}"
    context.run(command)


@task
def test(context: Context):
    """Run Python, client and both real-backend browser suites."""
    context.run("uv run pytest --cov=phrase_gym --cov-report=term-missing")
    context.run("npm --prefix frontend run typecheck")
    context.run("npm --prefix frontend test")
    build_frontend(context)
    context.run("npm --prefix frontend run test:browser")


namespace = Collection.from_module(sys.modules[__name__])
for name in ALLOWED_VERSION_TYPES:
    namespace.add_task(ver_task_factory(name), name=f"ver-{name}")
for name in ALLOWED_DOC_LANGUAGES:
    namespace.add_task(docs_task_factory(name), name=f"docs-{name}")
