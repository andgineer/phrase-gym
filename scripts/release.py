import argparse
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VERSION_FILE = "src/phrase_gym/__about__.py"
SEMVER = re.compile(r"v(\d+)\.(\d+)\.(\d+)")


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def package_version() -> str:
    match = re.search(r'__version__ = "(\d+\.\d+\.\d+)"', (ROOT / VERSION_FILE).read_text())
    if match is None:
        raise RuntimeError("The package version must be a numeric major.minor.patch")
    return match[1]


def validate_tag(tag: str) -> str:
    if not SEMVER.fullmatch(tag):
        raise RuntimeError("Package releases require vMAJOR.MINOR.PATCH tags")
    version = package_version()
    if tag != f"v{version}":
        raise RuntimeError("Tag does not match the package version")
    return version


def next_version(current: str, kind: str) -> str:
    major, minor, patch = map(int, current.split("."))
    if kind == "release":
        return f"{major + 1}.0.0"
    if kind == "feature":
        return f"{major}.{minor + 1}.0"
    if kind == "bug":
        return f"{major}.{minor}.{patch + 1}"
    raise ValueError(f"Unknown version increment: {kind}")


def release(kind: str, *, dry_run: bool = False, short_tag: bool = False) -> None:
    if git("status", "--porcelain"):
        raise RuntimeError("Commit and push all changes before setting a version tag")
    if git("branch", "--show-current") != "main":
        raise RuntimeError("Create releases from main")
    if not dry_run:
        git("fetch", "origin", "main", "--tags")
    if git("rev-parse", "HEAD") != git("rev-parse", "origin/main"):
        raise RuntimeError("Local main must match origin/main before releasing")
    versions = [package_version()]
    for tag in git("tag", "--list").splitlines():
        if match := SEMVER.fullmatch(tag):
            versions.append(".".join(match.groups()))
    latest = max(versions, key=lambda value: tuple(map(int, value.split("."))))
    if kind == "ss":
        print(f"{latest}.dev0+g{git('rev-parse', '--short', 'HEAD')}")
        return
    version = next_version(latest, kind)
    tag_version = version
    if short_tag:
        tag_version = ".".join(version.split(".")[: {"release": 1, "feature": 2, "bug": 3}[kind]])
    tag = f"v{tag_version}"
    print(f"{latest} -> {version} ({tag})")
    if dry_run or input("Commit and push this release? [y/N] ").lower() not in {"y", "yes"}:
        return
    (ROOT / VERSION_FILE).write_text(f'__version__ = "{version}"\n')
    subprocess.run(["uv", "lock"], cwd=ROOT, check=True)
    subprocess.run(["uv", "run", "inv", "pre"], cwd=ROOT, check=True)
    subprocess.run(["uv", "run", "inv", "test"], cwd=ROOT, check=True)
    git("add", VERSION_FILE, "uv.lock")
    git("commit", "-m", f"Version {tag}")
    git("tag", "-a", tag, "-m", f"Release {version}")
    git("push", "--atomic", "origin", "main", tag)


def main() -> None:
    parser = argparse.ArgumentParser(description="Version and release Phrase Gym")
    parser.add_argument("kind", choices=["release", "feature", "bug", "ss"])
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--short-tag", action="store_true")
    args = parser.parse_args()
    try:
        release(args.kind, dry_run=args.dry_run, short_tag=args.short_tag)
    except (RuntimeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
