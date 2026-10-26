import subprocess
from pathlib import Path

import pytest

from scripts import release


@pytest.fixture
def repository(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setattr(release, "ROOT", tmp_path)
    version_file = tmp_path / release.VERSION_FILE
    version_file.parent.mkdir(parents=True)
    version_file.write_text('__version__ = "0.1.0"\n')
    responses = {
        ("status", "--porcelain"): "",
        ("branch", "--show-current"): "main",
        ("rev-parse", "HEAD"): "abc123",
        ("rev-parse", "origin/main"): "abc123",
        ("rev-parse", "--short", "HEAD"): "abc123",
        ("tag", "--list"): "v0.9.0\nv0.10.0\nv0.2.0\nunrelated\nv99.0.0-rc1",
    }
    commands: list[tuple[str, ...]] = []

    def git(*args: str) -> str:
        commands.append(("git", *args))
        return responses.get(args, "")

    def run(command, **_kwargs):
        commands.append(tuple(command))

    monkeypatch.setattr(release, "git", git)
    monkeypatch.setattr(release.subprocess, "run", run)
    monkeypatch.setattr("builtins.input", lambda _: "yes")
    return version_file, responses, commands


@pytest.mark.parametrize(
    ("kind", "expected"), [("release", "3.0.0"), ("feature", "2.5.0"), ("bug", "2.4.10")]
)
def test_semantic_version_increments(kind: str, expected: str) -> None:
    assert release.next_version("2.4.9", kind) == expected


def test_invalid_increment_is_rejected() -> None:
    with pytest.raises(ValueError, match="Unknown"):
        release.next_version("1.0.0", "typo")


@pytest.mark.parametrize("tag", ["v0.1", "0.1.0", "v0.1.0-rc1", "v0.1.1"])
def test_publication_rejects_nonrelease_and_mismatched_tags(repository, tag) -> None:
    with pytest.raises(RuntimeError):
        release.validate_tag(tag)


def test_publication_accepts_the_exact_package_tag(repository) -> None:
    assert release.validate_tag("v0.1.0") == "0.1.0"


def test_declining_release_does_not_change_version_or_push(repository, monkeypatch) -> None:
    version_file, _, commands = repository
    monkeypatch.setattr("builtins.input", lambda _: "no")
    release.release("bug")
    assert version_file.read_text() == '__version__ = "0.1.0"\n'
    assert not any(command[1] in {"commit", "push"} for command in commands)


def test_release_updates_lock_checks_then_pushes_branch_and_tag_atomically(repository) -> None:
    version_file, _, commands = repository
    release.release("bug")
    assert version_file.read_text() == '__version__ = "0.10.1"\n'
    assert ("uv", "lock") in commands
    pre = commands.index(("uv", "run", "python", "scripts/checks.py", "pre"))
    test = commands.index(("uv", "run", "python", "scripts/checks.py", "test"))
    commit = commands.index(("git", "commit", "-m", "Version v0.10.1"))
    assert pre < test < commit
    assert commands[-2:] == [
        ("git", "tag", "-a", "v0.10.1", "-m", "Release 0.10.1"),
        ("git", "push", "--atomic", "origin", "main", "v0.10.1"),
    ]


@pytest.mark.parametrize(
    ("command", "response", "message"),
    [
        (("status", "--porcelain"), " M file", "Commit and push"),
        (("branch", "--show-current"), "feature", "from main"),
        (("rev-parse", "origin/main"), "behind", "match origin/main"),
    ],
)
def test_dirty_diverged_or_non_main_tree_cannot_release(repository, command, response, message):
    version_file, responses, commands = repository
    responses[command] = response
    with pytest.raises(RuntimeError, match=message):
        release.release("bug")
    assert version_file.read_text() == '__version__ = "0.1.0"\n'
    assert not any(command[:2] == ("git", "push") for command in commands)


def test_dry_run_has_no_writes_fetch_or_push(repository, capsys) -> None:
    version_file, _, commands = repository
    release.release("feature", dry_run=True)
    assert "0.10.0 -> 0.11.0 (v0.11.0)" in capsys.readouterr().out
    assert version_file.read_text() == '__version__ = "0.1.0"\n'
    assert all(command[1] in {"status", "branch", "rev-parse", "tag"} for command in commands)


def test_short_tag_helper_keeps_full_package_version(repository) -> None:
    version_file, _, commands = repository
    release.release("feature", short_tag=True)
    assert version_file.read_text() == '__version__ = "0.11.0"\n'
    assert commands[-1] == ("git", "push", "--atomic", "origin", "main", "v0.11")


def test_failed_gate_never_commits_or_tags(repository, monkeypatch) -> None:
    _, _, commands = repository

    def fail(command, **_kwargs):
        raise subprocess.CalledProcessError(1, command)

    monkeypatch.setattr(release.subprocess, "run", fail)
    with pytest.raises(subprocess.CalledProcessError):
        release.release("bug")
    assert not any(command[1] in {"commit", "push"} for command in commands)
    assert not any(command[:3] == ("git", "tag", "-a") for command in commands)


def test_snapshot_is_not_a_published_release(repository, capsys) -> None:
    version_file, _, commands = repository
    release.release("ss")
    assert capsys.readouterr().out.strip() == "0.10.0.dev0+gabc123"
    assert version_file.read_text() == '__version__ = "0.1.0"\n'
    assert not any(command[1] == "push" for command in commands)
