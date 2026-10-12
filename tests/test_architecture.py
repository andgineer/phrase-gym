from scripts.check_architecture import check_tree


def test_guard_reports_reverse_dependencies_local_imports_and_large_modules(tmp_path):
    domain = tmp_path / "src/phrase_gym/content"
    domain.mkdir(parents=True)
    (domain / "bad.py").write_text(
        "from phrase_gym.api.app import create_app\ndef inner():\n    import json\n",
        encoding="utf-8",
    )
    (domain / "large.py").write_text("# line\n" * 301, encoding="utf-8")
    errors = check_tree(tmp_path)
    assert any("cannot depend on HTTP" in error for error in errors)
    assert any("module top level" in error for error in errors)
    assert any("exceeds 300" in error for error in errors)


def test_guard_accepts_domain_contracts_and_excludes_generated_assets(tmp_path):
    domain = tmp_path / "src/phrase_gym/content"
    domain.mkdir(parents=True)
    (domain / "good.py").write_text(
        "from phrase_gym.contracts.content import ManualPackage\n", encoding="utf-8"
    )
    client = tmp_path / "frontend/src"
    client.mkdir(parents=True)
    (client / "api.generated.ts").write_text("// generated\n" * 301, encoding="utf-8")
    assert check_tree(tmp_path) == []
