#[cfg(unix)]
use std::os::unix::fs::symlink;

use super::*;

fn paths(root: &Path) -> CoachPaths {
    CoachPaths {
        config: root.join("config"),
        cache: root.join("cache"),
        logs: root.join("logs"),
        state: root.join("state"),
    }
}

#[test]
fn accepts_only_dedicated_uncustomized_home() {
    let root = tempfile::tempdir().unwrap();
    let paths = paths(root.path());
    let host = HostEnvironment::default();
    assert!(checked_home(&paths, &host).unwrap_err().contains("setup"));
    paths.ensure().unwrap();
    assert_eq!(
        checked_home(&paths, &host).unwrap(),
        paths.codex_home().canonicalize().unwrap()
    );
    for name in ["AGENTS.md", "AGENTS.override.md", "skills/custom/SKILL.md"] {
        let file = paths.codex_home().join(name);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "synthetic customization").unwrap();
        assert!(checked_home(&paths, &host).is_err(), "{name}");
        std::fs::remove_file(file).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn rejects_home_state_and_skills_symlinks_including_dangling_instructions() {
    let root = tempfile::tempdir().unwrap();
    let paths = paths(root.path());
    paths.ensure().unwrap();
    let host = HostEnvironment::default();
    symlink(
        root.path().join("missing"),
        paths.codex_home().join("AGENTS.md"),
    )
    .unwrap();
    assert!(checked_home(&paths, &host).is_err());
    std::fs::remove_file(paths.codex_home().join("AGENTS.md")).unwrap();
    for path in [
        paths.codex_home().join("skills"),
        paths.codex_home(),
        paths.state.clone(),
    ] {
        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }
        symlink(root.path(), &path).unwrap();
        assert!(checked_home(&paths, &host).is_err());
        std::fs::remove_file(path).unwrap();
        paths.ensure().unwrap();
    }
}

#[test]
fn pins_version_subscription_and_allowlisted_environment_without_model_invocation() {
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("codex");
    let home = root.path().join("owned home");
    let host = HostEnvironment::from_variables([
        ("OPENAI_API_KEY".into(), "synthetic-secret".into()),
        ("CODEX_HOME".into(), "operator-home".into()),
    ]);
    for (version, status, accepted, count) in [
        ("codex-cli 0.159.2", "Logged in using ChatGPT", true, 2),
        ("codex-cli 0.159.3", "Logged in using ChatGPT", false, 1),
        (
            "codex-cli 0.159.2",
            "Logged in using an API key: synthetic-secret",
            false,
            2,
        ),
        ("codex-cli 0.159.2", "Not logged in", false, 2),
    ] {
        let mut calls = 0;
        let result = check_with(&executable, &home, &host, root.path(), |spec| {
            calls += 1;
            assert!(!spec.env.iter().any(|(key, _)| key == "OPENAI_API_KEY"));
            assert!(
                spec.env
                    .iter()
                    .any(|(key, value)| key == "CODEX_HOME" && value == home.as_os_str())
            );
            assert_eq!(spec.limits.output_bytes, 16384);
            let args: Vec<_> = spec.args.iter().map(|a| a.to_str().unwrap()).collect();
            match args.as_slice() {
                ["--version"] => Ok(version.into()),
                ["--ignore-user-config", "login", "status"] => Ok(status.into()),
                _ => panic!("unexpected command"),
            }
        });
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(calls, count);
        if let Err(error) = result {
            assert!(!error.contains("synthetic-secret"));
        }
    }
}

#[test]
fn probe_failure_does_not_expose_or_retain_output() {
    let root = tempfile::tempdir().unwrap();
    let spec = ProcessSpec {
        program: root.path().join("missing-native"),
        args: vec![],
        cwd: root.path().into(),
        env: vec![],
        stdout: root.path().join("stdout"),
        stderr: root.path().join("stderr"),
        limits: Limits {
            output_bytes: 100,
            deadline: Duration::from_secs(1),
            grace: Duration::ZERO,
        },
    };
    assert!(probe(spec).is_err());
    assert!(!root.path().join("stdout").exists());
    assert!(!root.path().join("stderr").exists());
}
