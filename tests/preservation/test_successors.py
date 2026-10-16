import re
import tomllib

from tests.integration.helpers import ROOT
from tests.preservation.scenarios import SCENARIOS, load, scenario_names

RECORD = ROOT / "fixtures/preservation/successors.toml"
STATUSES = {"covered", "pending", "superseded"}
SCENARIO_RUNNER = "rust:crates/phrase-gym-server/tests/api/preservation.rs"
RUST_TEST_MARKERS = ("#[test]", "#[tokio::test")
EXPORTER = "tests.preservation.export"
SCRIPT_IMPORT = re.compile(r"^(?:from|import) scripts\b", re.MULTILINE)


def modules() -> list[dict]:
    return tomllib.loads(RECORD.read_text(encoding="utf-8"))["module"]


def implementations() -> dict[str, list[str]]:
    return {name: load(name)["implementations"] for name in scenario_names()}


def legacy_test_modules() -> set[str]:
    return {
        path.relative_to(ROOT).as_posix()
        for path in (ROOT / "tests").rglob("*.py")
        if path.name.startswith("test_")
        or (path.parent == ROOT / "tests" and path.name.startswith("browser_"))
    }


def resolves(reference: str) -> bool:
    kind, _, target = reference.partition(":")
    if kind == "scenario":
        return (SCENARIOS / f"{target}.json").is_file()
    return kind in {"rust", "client"} and (ROOT / target).is_file()


def coverage_problems(entry: dict, declared: dict[str, list[str]]) -> list[str]:
    successors = entry.get("successors", [])
    references = [reference.partition(":") for reference in successors]
    named = [target for kind, _, target in references if kind == "scenario"]
    # Citing the scenario runner without naming scenarios relies on the whole scenario suite.
    relied_on = named or (sorted(declared) if SCENARIO_RUNNER in successors else [])
    problems = [] if successors else ["names no successor"]
    problems += [
        f"scenario {name} is not declared for rust"
        for name in relied_on
        if "rust" not in declared[name]
    ]
    for kind, _, target in references:
        source = ROOT / target
        if kind == "rust" and not any(
            mark in source.read_text(encoding="utf-8") for mark in RUST_TEST_MARKERS
        ):
            problems.append(f"{target} contains no Rust test")
        if kind == "client" and not target.startswith("frontend/tests/"):
            problems.append(f"{target} is not a client test")
    return problems


def test_every_legacy_test_module_has_exactly_one_successor_entry():
    paths = [entry["path"] for entry in modules()]
    assert len(paths) == len(set(paths))
    assert set(paths) == legacy_test_modules() | {"tests/preservation/test_successors.py"}


def test_successor_entries_are_complete_and_resolvable():
    declared = implementations()
    for entry in modules():
        assert entry["status"] in STATUSES, entry
        assert entry["responsibility"], entry
        assert all(resolves(reference) for reference in entry.get("successors", [])), entry
        if entry["status"] == "covered":
            assert coverage_problems(entry, declared) == [], entry
        else:
            assert entry["stage"] in range(2, 8), entry
        if entry["status"] == "superseded":
            assert (ROOT / entry["decision"]).is_file() and entry["replacement"], entry


def test_python_fixture_drift_checks_end_at_cutover():
    for entry in modules():
        if EXPORTER in (ROOT / entry["path"]).read_text(encoding="utf-8"):
            assert (entry["status"], entry["stage"]) == ("superseded", 7), entry


def test_python_script_checks_end_at_cutover():
    # Project scripts guard or run live Python code until the cutover; nothing covers them earlier.
    for entry in modules():
        if SCRIPT_IMPORT.search((ROOT / entry["path"]).read_text(encoding="utf-8")):
            assert entry["status"] != "covered" and entry["stage"] == 7, entry


def test_coverage_requires_rust_execution_of_every_relied_on_scenario():
    declared = {"ported": ["python", "rust"], "legacy": ["python"]}
    assert coverage_problems({"successors": ["scenario:ported", SCENARIO_RUNNER]}, declared) == []
    assert coverage_problems({"successors": [SCENARIO_RUNNER]}, declared) == [
        "scenario legacy is not declared for rust",
    ]
    assert coverage_problems({"successors": ["scenario:legacy"]}, declared) == [
        "scenario legacy is not declared for rust",
    ]
    assert coverage_problems({}, declared) == ["names no successor"]


def test_coverage_requires_rust_tests_and_client_tests():
    declared = {"ported": ["python", "rust"]}
    assert coverage_problems(
        {"successors": ["rust:crates/phrase-gym-core/src/lib.rs"]}, declared
    ) == [
        "crates/phrase-gym-core/src/lib.rs contains no Rust test",
    ]
    assert coverage_problems({"successors": ["client:frontend/src/api.ts"]}, declared) == [
        "frontend/src/api.ts is not a client test",
    ]
    assert coverage_problems({"successors": ["rust:xtask/src/modules.rs"]}, declared) == []
