import shlex
import shutil
import sys
from contextlib import contextmanager
from pathlib import Path

from invoke import Collection, Context, task

from phrase_gym.__about__ import __version__

DOCS_PATH = Path("docs")
DOCS_SRC_PATH = DOCS_PATH / "src"
ALLOWED_DOC_LANGUAGES = ["en"] + sorted(
    path.name for path in DOCS_SRC_PATH.iterdir() if path.is_dir() and path.name != "en"
)
ALLOWED_VERSION_TYPES = ["release", "bug", "feature"]


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
    shutil.copytree(DOCS_PATH / "common", source_path, dirs_exist_ok=True)
    shutil.copyfile(DOCS_PATH / "importing-phrases.md", source_path / "importing-phrases.md")
    shutil.copyfile(DOCS_PATH / "development.md", source_path / "development.md")
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


namespace = Collection.from_module(sys.modules[__name__])
for name in ALLOWED_VERSION_TYPES:
    namespace.add_task(ver_task_factory(name), name=f"ver-{name}")
for name in ALLOWED_DOC_LANGUAGES:
    namespace.add_task(docs_task_factory(name), name=f"docs-{name}")
