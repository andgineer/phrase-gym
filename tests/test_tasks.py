from pathlib import Path

import tasks


def test_documentation_rendering_keeps_shared_guides_and_language_output_separate(
    tmp_path, monkeypatch
):
    monkeypatch.chdir(tmp_path)
    docs = Path("docs")
    (docs / "src/en").mkdir(parents=True)
    (docs / "src/de").mkdir()
    (docs / "common").mkdir()
    (docs / "mkdocs.yml").write_text("docs_dir: src/LANGUAGE\nsite_dir: SITE_DIR\n")
    (docs / "src/en/index.md").write_text("English overview")
    (docs / "src/de/index.md").write_text("Other overview")
    (docs / "common/reference.md").write_text("Shared reference")
    (docs / "importing-phrases.md").write_text("Import guide")
    (docs / "development.md").write_text("Release guide")
    with tasks.docs_rendered("en") as config:
        assert "site_dir: site\n" in config.read_text()
        assert Path("build/docs/src/en/importing-phrases.md").read_text() == "Import guide"
    with tasks.docs_rendered("de") as config:
        assert "site_dir: site/de\n" in config.read_text()
        assert Path("build/docs/src/de/reference.md").read_text() == "Shared reference"
        assert Path("build/docs/src/de/development.md").read_text() == "Release guide"
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
    } <= set(tasks.namespace.task_names)
