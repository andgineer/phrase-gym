"""Keep dependency direction and module size visible as the application grows."""

import ast
from pathlib import Path

MAX_SOURCE_LINES = 300
SOURCE_EXTENSIONS = {".py", ".ts", ".vue"}
DOMAIN_DIRECTORIES = {"content", "media", "contracts", "sessions", "practice"}


def check_imports(path: Path, package: Path) -> list[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"))
    top_level = {id(node) for node in tree.body}
    family = path.relative_to(package).parts[0]
    errors = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Import | ast.ImportFrom):
            continue
        if id(node) not in top_level:
            errors.append(f"{path}:{node.lineno}: imports must stay at module top level")
        modules = (
            [node.module or ""]
            if isinstance(node, ast.ImportFrom)
            else [alias.name for alias in node.names]
        )
        for module in modules:
            if module == "__future__":
                errors.append(f"{path}:{node.lineno}: future annotations are not used here")
            if family in DOMAIN_DIRECTORIES and module.startswith("phrase_gym.api"):
                errors.append(f"{path}:{node.lineno}: domain code cannot depend on HTTP")
            if (
                family == "contracts"
                and module.startswith("phrase_gym.")
                and not module.startswith(
                    "phrase_gym.contracts",
                )
            ):
                errors.append(f"{path}:{node.lineno}: contracts cannot depend on services")
    return errors


def check_tree(root: Path) -> list[str]:
    package = root / "src/phrase_gym"
    frontend = root / "frontend/src"
    errors = []
    for directory in (package, frontend):
        for path in sorted(directory.rglob("*")):
            if path.suffix not in SOURCE_EXTENSIONS or "static" in path.parts:
                continue
            if "migrations" in path.parts or path.name.endswith(".generated.ts"):
                continue
            if len(path.read_text(encoding="utf-8").splitlines()) > MAX_SOURCE_LINES:
                errors.append(
                    f"{path}: exceeds {MAX_SOURCE_LINES} lines; split responsibilities",
                )
            if path.suffix == ".py":
                errors.extend(check_imports(path, package))
    return errors


def main() -> None:
    errors = check_tree(Path(__file__).resolve().parents[1])
    if errors:
        raise SystemExit("\n".join(errors))
    print("Architecture checks passed")


if __name__ == "__main__":
    main()
