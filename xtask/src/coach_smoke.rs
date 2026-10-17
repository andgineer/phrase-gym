use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::coach::{BINARY, bundle, io};
use crate::tools::{Failure, Workspace};

const TERMINAL_CANARY: &str = "PHRASE_GYM_TERMINAL_CANARY";
// Injected into the app's own launch environment with fake values; the app must keep them from the
// bundled fake CLI. Kept in step with the app's smoke checks.
const LAUNCH_VARIABLES: [(&str, &str); 3] = [
    ("PHRASE_GYM_LAUNCH_CANARY", "1"),
    ("ANTHROPIC_API_KEY", "sk-ant-fake-coach-smoke"),
    ("OPENAI_API_KEY", "sk-fake-coach-smoke"),
];
const LAUNCH_DEADLINE: Duration = Duration::from_mins(4);

// Launches the bundle the way a user does (LaunchServices on macOS, a fresh user environment on
// Windows), then checks the report the app wrote and the renderer-failure path.
pub fn smoke(workspace: &Workspace) -> Result<(), Failure> {
    let out = workspace.root.join("target/coach-smoke/Coach Smoke Ünï Ђ");
    if out.exists() {
        std::fs::remove_dir_all(&out).map_err(io("clearing the previous smoke bundle"))?;
    }
    let launcher = bundle(workspace, &out)?;
    let report = out.join("smoke report Ђ.json");
    wait(&mut user_launch(&launcher, &report)?)?;
    let document = read_report(&report)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&document).unwrap_or_default()
    );
    let mut problems = Vec::new();
    if document["passed"] != serde_json::Value::Bool(true) {
        problems.push("the packaged smoke report did not pass".to_owned());
    }
    if document["terminal_canary_inherited"] != serde_json::Value::Bool(false) {
        problems.push("the app inherited the launching terminal's environment".to_owned());
    }
    let injected: Vec<&str> = LAUNCH_VARIABLES.iter().map(|(name, _)| *name).collect();
    if document["launch_variables"] != serde_json::json!(injected) {
        problems.push(
            "the injected launch variables did not all reach the app, so the fake CLI's \
             environment check proved nothing"
                .to_owned(),
        );
    }
    let failing = out.join("renderer failure report.json");
    let status = Command::new(binary_inside(&launcher))
        .args(["--smoke-test".as_ref(), failing.as_os_str()])
        .env("WGPU_BACKEND", "gl")
        .status()
        .map_err(io("launching Coach with no usable renderer"))?;
    let failure = read_report(&failing)?;
    let message = failure["startup_error"].as_str().unwrap_or_default();
    println!("xtask: renderer failure exit {status}: {message}");
    if status.code() != Some(2) || !message.contains("graphics renderer") {
        problems
            .push("a missing renderer did not produce the actionable startup failure".to_owned());
    }
    if let Some(lock) = document["instance_lock"].as_str() {
        problems.extend(second_instance(&launcher, Path::new(lock), &out)?);
    } else {
        problems.push("the smoke report names no instance lock".to_owned());
    }
    if problems.is_empty() {
        println!("xtask: Coach packaged smoke test passed");
        Ok(())
    } else {
        Err(Failure::new(problems.join("\n")))
    }
}

// Holds Coach's own instance lock, as a running Coach would, and expects a second launch to refuse.
fn second_instance(launcher: &Path, lock: &Path, out: &Path) -> Result<Vec<String>, Failure> {
    let held = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(lock)
        .map_err(io("opening the Coach instance lock"))?;
    held.try_lock()
        .map_err(|error| Failure::new(format!("cannot hold the Coach instance lock: {error}")))?;
    let report = out.join("second instance report.json");
    let status = Command::new(binary_inside(launcher))
        .args(["--smoke-test".as_ref(), report.as_os_str()])
        .status()
        .map_err(io("launching a second Coach"))?;
    drop(held);
    let refused = read_report(&report)?;
    let message = refused["startup_error"].as_str().unwrap_or_default();
    println!("xtask: second instance exit {status}: {message}");
    if status.code() == Some(3) && message.contains("already running") {
        Ok(Vec::new())
    } else {
        Ok(vec![
            "a second Coach was not refused by the instance lock".to_owned(),
        ])
    }
}

fn binary_inside(launcher: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        launcher.join(format!("Contents/MacOS/{BINARY}"))
    } else {
        launcher.to_path_buf()
    }
}

fn user_launch(launcher: &Path, report: &Path) -> Result<Command, Failure> {
    if cfg!(target_os = "macos") {
        // `open` forwards its own environment to the app, so it gets only what launchd gives a
        // Finder launch plus the variables added with --env.
        let mut command = Command::new("/usr/bin/open");
        command.env_clear();
        for name in ["HOME", "USER", "LOGNAME", "SHELL", "TMPDIR"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        command.env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");
        command.arg("-n").arg("-W");
        for (name, value) in LAUNCH_VARIABLES {
            command.arg("--env").arg(format!("{name}={value}"));
        }
        command.arg(launcher);
        command.arg("--args").arg("--smoke-test").arg(report);
        Ok(command)
    } else if cfg!(windows) {
        if Command::new("pwsh").arg("-Version").output().is_err() {
            return Err(Failure::new(
                "coach-smoke on Windows needs PowerShell 7 (pwsh) to start Coach",
            ));
        }
        let mut command = Command::new("pwsh");
        let script = windows_launch_script(launcher, report);
        command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
        // Visible to PowerShell only; the rebuilt environment must keep it from the app.
        command.env(TERMINAL_CANARY, "1");
        Ok(command)
    } else {
        Err(Failure::new("Coach smoke tests run on macOS or Windows"))
    }
}

// Rebuilds the environment as Start-Process -UseNewEnvironment does (machine and user variables,
// merged Path), then adds the injected ones, which that cmdlet's -Environment silently dropped.
fn windows_launch_script(launcher: &Path, report: &Path) -> String {
    let quote = |text: String| format!("'{}'", text.replace('\'', "''"));
    let mut lines = vec![
        format!(
            "$psi = [System.Diagnostics.ProcessStartInfo]::new({})",
            quote(launcher.display().to_string())
        ),
        "$psi.UseShellExecute = $false".to_owned(),
        "$psi.ArgumentList.Add('--smoke-test')".to_owned(),
        format!(
            "$psi.ArgumentList.Add({})",
            quote(report.display().to_string())
        ),
        "$psi.Environment.Clear()".to_owned(),
        "foreach ($scope in 'Machine', 'User') { $vars = [Environment]::GetEnvironmentVariables($scope); \
         foreach ($name in $vars.Keys) { $psi.Environment[$name] = $vars[$name] } }"
            .to_owned(),
        "$psi.Environment['Path'] = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + \
         [Environment]::GetEnvironmentVariable('Path', 'User')"
            .to_owned(),
    ];
    lines.extend(
        LAUNCH_VARIABLES
            .iter()
            .map(|(name, value)| format!("$psi.Environment['{name}'] = '{value}'")),
    );
    lines.push(
        "$p = [System.Diagnostics.Process]::Start($psi); $p.WaitForExit(); exit $p.ExitCode"
            .to_owned(),
    );
    lines.join("\n")
}

fn wait(command: &mut Command) -> Result<(), Failure> {
    let mut child = command.spawn().map_err(io("launching the Coach bundle"))?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(io("waiting for Coach"))? {
            println!("xtask: Coach launcher exited with {status}");
            return Ok(());
        }
        if started.elapsed() > LAUNCH_DEADLINE {
            let _ = child.kill();
            return Err(Failure::new("Coach did not finish its smoke test in time"));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn read_report(path: &Path) -> Result<serde_json::Value, Failure> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| Failure::new(format!("no smoke report at {}: {error}", path.display())))?;
    serde_json::from_str(&text)
        .map_err(|error| Failure::new(format!("invalid smoke report: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_windows_launch_rebuilds_the_environment_and_injects_the_variables() {
        let script = windows_launch_script(
            Path::new(r"D:\Coach Smoke Ünï Ђ\Phrase Gym Coach\phrase-gym-coach.exe"),
            Path::new(r"D:\Coach Smoke Ünï Ђ\it's report.json"),
        );
        let clear = script.find("$psi.Environment.Clear()").unwrap();
        let machine_and_user = script.find("GetEnvironmentVariables($scope)").unwrap();
        assert!(clear < machine_and_user);
        for (name, value) in LAUNCH_VARIABLES {
            let injected = script
                .find(&format!("$psi.Environment['{name}'] = '{value}'"))
                .unwrap();
            assert!(machine_and_user < injected, "{name}");
        }
        assert!(
            script.contains(r"::new('D:\Coach Smoke Ünï Ђ\Phrase Gym Coach\phrase-gym-coach.exe')")
        );
        assert!(script.contains(r"ArgumentList.Add('D:\Coach Smoke Ünï Ђ\it''s report.json')"));
        assert!(!script.contains("UseNewEnvironment"));
        assert!(script.ends_with("exit $p.ExitCode"));
    }
}
