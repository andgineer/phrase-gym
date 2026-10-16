import pytest

from tests.preservation.scenarios import load, run, scenario_names


@pytest.mark.parametrize("name", scenario_names())
def test_python_server_preserves_scenario(name, tmp_path):
    run(name, tmp_path / "data")


def test_scenarios_are_declared_for_python_and_cover_each_preserved_area():
    names = scenario_names()
    assert all("python" in load(name)["implementations"] for name in names)
    assert {
        "publication",
        "session-events",
        "practice-repairs",
        "historical-ratings",
        "recordings",
        "reference-media",
        "media-reconciliation",
    } <= set(names)
