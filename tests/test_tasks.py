from pathlib import Path

import pytest
from invoke import Context, Result

import tasks


def test_documentation_rendering_uses_each_language_source_without_injected_pages(
    tmp_path, monkeypatch
):
    monkeypatch.chdir(tmp_path)
    docs = Path("docs")
    (docs / "src/en").mkdir(parents=True)
    (docs / "src/de").mkdir()
    (docs / "mkdocs.yml").write_text("docs_dir: src/LANGUAGE\nsite_dir: SITE_DIR\n")
    (docs / "src/en/index.md").write_text("English overview")
    (docs / "src/de/index.md").write_text("Other overview")
    (docs / "src/en/importing-phrases.md").write_text("Import guide")
    (docs / "src/de/reference.md").write_text("Localized reference")
    (docs / "src/en/backup-and-restore.md").write_text("Backup guide")
    stale = Path("build/docs/src/en/development.md")
    stale.parent.mkdir(parents=True)
    stale.write_text("Stale development page")
    with tasks.docs_rendered("en") as config:
        assert "site_dir: site\n" in config.read_text()
        assert not stale.exists()
        assert Path("build/docs/src/en/backup-and-restore.md").read_text() == "Backup guide"
        assert Path("build/docs/src/en/importing-phrases.md").read_text() == "Import guide"
    with tasks.docs_rendered("de") as config:
        assert "site_dir: site/de\n" in config.read_text()
        assert Path("build/docs/src/de/reference.md").read_text() == "Localized reference"
        assert not Path("build/docs/src/de/importing-phrases.md").exists()
        assert not Path("build/docs/src/de/development.md").exists()
    assert (docs / "mkdocs.yml").read_text().endswith("site_dir: SITE_DIR\n")
    assert Path("build/docs/src/en/index.md").read_text() == "English overview"


def test_scaffold_commands_remain_available() -> None:
    assert {
        "version",
        "ver-release",
        "ver-feature",
        "ver-bug",
        "pre",
        "reqs",
        "uv",
        "docs-en",
        "build-docs",
        "build",
        "build-frontend",
        "run",
        "test",
    } <= set(tasks.namespace.task_names)


@pytest.mark.parametrize("build", [False, True])
def test_server_builds_frontend_only_when_requested(monkeypatch, build):
    commands = []
    context = Context()
    monkeypatch.setattr(context, "run", commands.append)
    tasks.run(context, build=build)
    assert commands == (
        ["npm --prefix frontend run build", "uv run phrase-gym run"]
        if build
        else ["uv run phrase-gym run"]
    )


def test_distribution_build_includes_the_client(monkeypatch):
    commands = []
    context = Context()
    monkeypatch.setattr(context, "run", commands.append)
    tasks.build(context)
    assert commands == ["npm --prefix frontend ci", "npm --prefix frontend run build", "uv build"]


def test_tailscale_run_routes_to_loopback_and_allows_only_the_machine_origin(monkeypatch):
    commands = []
    context = Context()

    def execute(command, **_kwargs):
        commands.append(command)
        return Result(stdout='{"Self":{"DNSName":"macbook.example.ts.net."}}')

    monkeypatch.setattr(context, "run", execute)
    tasks.run(context, tailscale=True)
    assert commands == [
        "tailscale status --json",
        "tailscale serve --bg --yes http://127.0.0.1:8000",
        "uv run phrase-gym run --tailscale-origin https://macbook.example.ts.net",
    ]


def test_size_check_includes_nested_modules_but_excludes_generated_code(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    source = Path("frontend/src/composables")
    source.mkdir(parents=True)
    (source / "api.generated.ts").write_text("line\n" * 301)
    tasks.check_sizes(Context())
    (source / "useLarge.ts").write_text("line\n" * 301)
    with pytest.raises(SystemExit, match="useLarge.ts: 301 lines"):
        tasks.check_sizes(Context())
