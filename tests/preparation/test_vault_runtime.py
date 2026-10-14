import pytest
from click.testing import CliRunner

from phrase_gym.main import phrase_gym
from tests.preparation.test_vault import manifest


@pytest.mark.parametrize("path_kind", ["nested", "symlink", "default", "vault_alias", "case_alias"])
def test_vault_export_rejects_runtime_inside_vault_before_any_write(
    tmp_path, monkeypatch, path_kind
):
    vault = tmp_path / "vault"
    vault.mkdir()
    note = vault / "lesson.md"
    note.write_text("- Right. ^a\n- Right. ^b\n")
    mapping = tmp_path / "manifest.json"
    mapping.write_text(manifest().model_dump_json())
    data_dir = vault / "app-data"
    selected_root = vault
    args = ["--data-dir", str(data_dir)]
    if path_kind == "case_alias":
        alias = tmp_path / "VAULT"
        if not alias.exists():
            alias.symlink_to(vault, target_is_directory=True)
        assert alias.samefile(vault)
        args = ["--data-dir", str(alias / "app-data")]
    elif path_kind == "symlink":
        alias = tmp_path / "alias"
        alias.symlink_to(vault, target_is_directory=True)
        args = ["--data-dir", str(alias / "app-data")]
    elif path_kind == "default":
        monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(data_dir))
        args = []
    elif path_kind == "vault_alias":
        selected_root = tmp_path / "vault-alias"
        selected_root.symlink_to(vault, target_is_directory=True)
    before = {
        str(path.relative_to(vault)): path.read_bytes()
        for path in vault.rglob("*")
        if path.is_file()
    }
    response = CliRunner().invoke(
        phrase_gym, [*args, "export-vault", str(selected_root), str(mapping)]
    )
    assert response.exit_code != 0
    assert "Runtime data directory must stay outside" in response.output
    assert not data_dir.exists()
    after = {
        str(path.relative_to(vault)): path.read_bytes()
        for path in vault.rglob("*")
        if path.is_file()
    }
    assert after == before
    assert sorted(path.name for path in vault.iterdir()) == ["lesson.md"]


def test_vault_export_rejects_output_case_alias_before_runtime_creation(tmp_path):
    vault = tmp_path / "vault"
    vault.mkdir()
    note = vault / "lesson.md"
    note.write_text("- Right. ^a\n- Right. ^b\n")
    mapping = tmp_path / "manifest.json"
    mapping.write_text(manifest().model_dump_json())
    alias = tmp_path / "VAULT"
    if not alias.exists():
        alias.symlink_to(vault, target_is_directory=True)
    assert alias.samefile(vault)
    data_dir = tmp_path / "safe-data"
    response = CliRunner().invoke(
        phrase_gym,
        [
            "--data-dir",
            str(data_dir),
            "export-vault",
            str(vault),
            str(mapping),
            "--output",
            str(alias / "export.json"),
        ],
    )
    assert response.exit_code != 0
    assert "Export output must stay outside" in response.output
    assert not data_dir.exists()
    assert sorted(path.name for path in vault.iterdir()) == ["lesson.md"]
    assert note.read_text() == "- Right. ^a\n- Right. ^b\n"
