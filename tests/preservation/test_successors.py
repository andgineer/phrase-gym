import json
import tomllib

from tests.integration.helpers import ROOT
from tests.preservation.scenarios import SCENARIOS

RECORD = ROOT / "fixtures/preservation/successors.toml"
STATUSES = {"covered", "pending", "superseded"}


def modules() -> list[dict]:
    return tomllib.loads(RECORD.read_text(encoding="utf-8"))["module"]


def legacy_test_modules() -> set[str]:
    return {
        path.relative_to(ROOT).as_posix()
        for path in (ROOT / "tests").rglob("*.py")
        if path.name.startswith("test_") or path.parent == ROOT / "tests" and path.name.startswith("browser_")
    }


def resolves(reference: str) -> bool:
    kind, _, target = reference.partition(":")
    if kind == "scenario":
        return (SCENARIOS / f"{target}.json").is_file()
    return kind in {"rust", "client"} and (ROOT / target).is_file()


def runs_without_python(reference: str) -> bool:
    kind, _, target = reference.partition(":")
    if kind == "scenario":
        scenario = json.loads((SCENARIOS / f"{target}.json").read_text(encoding="utf-8"))
        return "rust" in scenario["implementations"]
    return kind in {"rust", "client"}


def test_every_legacy_test_module_has_exactly_one_successor_entry():
    paths = [entry["path"] for entry in modules()]
    assert len(paths) == len(set(paths))
    assert set(paths) == legacy_test_modules() | {"tests/preservation/test_successors.py"}


def test_successor_entries_are_complete_and_resolvable():
    for entry in modules():
        assert entry["status"] in STATUSES, entry
        assert entry["responsibility"], entry
        successors = entry.get("successors", [])
        assert all(resolves(reference) for reference in successors), entry
        if entry["status"] == "covered":
            assert any(runs_without_python(reference) for reference in successors), entry
        else:
            assert entry["stage"] in range(2, 8), entry
        if entry["status"] == "superseded":
            assert (ROOT / entry["decision"]).is_file() and entry["replacement"], entry
