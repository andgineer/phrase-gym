// A stand-in for agent CLIs in supervision tests and packaged smoke checks; a native executable so
// the same scenarios run on Windows, where POSIX scripts do not.
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

// Bounds every scenario so a leaked process from a failed test eventually exits by itself.
const LIFETIME: Duration = Duration::from_mins(2);

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["report", path, rest @ ..] => report(Path::new(path), rest),
        ["flood", bytes] => flood(bytes.parse().unwrap_or(0)),
        ["sleep", millis] => pause(millis),
        ["ignore-term", millis] => ignore_term(millis),
        ["grandchild", heartbeat, then] => grandchild(heartbeat, then),
        ["heartbeat", path] => heartbeat(Path::new(path)),
        ["exit", code, message] => {
            eprintln!("{message}");
            ExitCode::from(code.parse::<u8>().unwrap_or(1))
        }
        _ => {
            eprintln!("usage: phrase-gym-fake-child <scenario> ...");
            ExitCode::from(2)
        }
    }
}

fn report(path: &Path, rest: &[&str]) -> ExitCode {
    let environment: serde_json::Map<String, serde_json::Value> = std::env::vars_os()
        .map(|(key, value)| {
            (
                key.to_string_lossy().into_owned(),
                value.to_string_lossy().into_owned().into(),
            )
        })
        .collect();
    let document = serde_json::json!({
        "args": rest,
        "cwd": std::env::current_dir().map(|cwd| cwd.to_string_lossy().into_owned()).ok(),
        "env": environment,
    });
    if std::fs::write(path, document.to_string()).is_err() {
        return ExitCode::from(3);
    }
    println!("reported");
    ExitCode::SUCCESS
}

fn flood(bytes: usize) -> ExitCode {
    let chunk = vec![b'x'; 8192];
    let mut stdout = std::io::stdout().lock();
    let mut remaining = bytes;
    while remaining > 0 {
        let size = remaining.min(chunk.len());
        if stdout.write_all(&chunk[..size]).is_err() {
            return ExitCode::from(4);
        }
        remaining -= size;
    }
    ExitCode::SUCCESS
}

fn pause(millis: &str) -> ExitCode {
    let wanted = Duration::from_millis(millis.parse().unwrap_or(0));
    std::thread::sleep(wanted.min(LIFETIME));
    ExitCode::SUCCESS
}

#[cfg(unix)]
fn ignore_term(millis: &str) -> ExitCode {
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    if signal_hook::flag::register(signal_hook::consts::SIGTERM, flag).is_err() {
        return ExitCode::from(5);
    }
    println!("armed");
    pause(millis)
}

#[cfg(not(unix))]
fn ignore_term(millis: &str) -> ExitCode {
    println!("armed");
    pause(millis)
}

fn grandchild(heartbeat: &str, then: &str) -> ExitCode {
    let Ok(executable) = std::env::current_exe() else {
        return ExitCode::from(6);
    };
    // Inherits stdout/stderr like a CLI's own tool processes, so it also holds the capture pipes.
    if Command::new(executable)
        .args(["heartbeat", heartbeat])
        .spawn()
        .is_err()
    {
        return ExitCode::from(7);
    }
    // Leave only once the grandchild is demonstrably alive, so cleanup is tested against a live one.
    let started = Instant::now();
    while !Path::new(heartbeat).exists() && started.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(10));
    }
    println!("spawned");
    if then == "wait" {
        std::thread::sleep(LIFETIME);
    }
    ExitCode::SUCCESS
}

fn heartbeat(path: &Path) -> ExitCode {
    let started = Instant::now();
    let mut beat: u64 = 0;
    while started.elapsed() < LIFETIME {
        beat += 1;
        let _ = std::fs::write(path, beat.to_string());
        std::thread::sleep(Duration::from_millis(20));
    }
    ExitCode::SUCCESS
}
