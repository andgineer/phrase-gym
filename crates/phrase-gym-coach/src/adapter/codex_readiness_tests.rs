use super::*;

#[test]
fn uses_normal_home_without_reading_credentials_or_rejecting_user_customizations() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join(".codex");
    std::fs::create_dir(&home).unwrap();
    std::fs::write(home.join("AGENTS.md"), "normal user customization").unwrap();
    let paths = CoachPaths {
        config: root.path().into(),
        cache: root.path().into(),
        logs: root.path().into(),
        state: root.path().into(),
    };
    let host =
        HostEnvironment::from_variables([("HOME".into(), root.path().as_os_str().to_owned())]);
    assert_eq!(
        checked_home(&paths, &host).unwrap(),
        home.canonicalize().unwrap()
    );
    assert!(!home.join("auth.json").exists());
    assert!(checked_home(&paths, &HostEnvironment::default()).is_err());
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
        ("codex-cli 0.163.0", "Logged in using ChatGPT", true, 2),
        ("codex-cli 0.163.1", "Logged in using ChatGPT", false, 1),
        (
            "codex-cli 0.163.0",
            "Logged in using an API key: synthetic-secret",
            false,
            2,
        ),
        ("codex-cli 0.163.0", "Not logged in", false, 2),
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
                ["login", "status"] => Ok(status.into()),
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
