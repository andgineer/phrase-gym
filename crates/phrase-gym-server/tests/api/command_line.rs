#[test]
fn the_server_reports_its_package_version() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_phrase-gym-server"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        format!("phrase-gym-server {}", env!("CARGO_PKG_VERSION"))
    );
}
