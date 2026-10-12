import json
from click.testing import CliRunner
from phrase_gym.main import phrase_gym
from tests.integration.helpers import ROOT


def test_cli_dry_run_stage_publish_and_export(tmp_path):
    runner = CliRunner()
    args = ["--data-dir", str(tmp_path / "data")]
    dry = runner.invoke(
        phrase_gym, [*args, "import-package", str(ROOT / "fixtures/manual-en.json")]
    )
    assert dry.exit_code == 0, dry.output
    assert json.loads(dry.output)["status"] == "validated"
    staged = runner.invoke(
        phrase_gym, [*args, "import-package", str(ROOT / "fixtures/manual-en.json"), "--stage"]
    )
    assert staged.exit_code == 0, staged.output
    import_id = json.loads(staged.output)["id"]
    assert runner.invoke(phrase_gym, [*args, "publish", import_id]).exit_code != 0
    assert runner.invoke(phrase_gym, [*args, "publish", import_id, "--reviewed"]).exit_code == 0
    schema_path = tmp_path / "schema.json"
    assert runner.invoke(phrase_gym, ["export-schema", str(schema_path)]).exit_code == 0
    assert json.loads(schema_path.read_text(encoding="utf-8"))["info"]["title"] == "Phrase Gym"
