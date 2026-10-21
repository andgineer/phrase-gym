use std::ffi::OsString;
use std::future::pending;
use std::path::{Path, PathBuf};
use std::time::Duration;

use phrase_gym_coach::adapter::environment::{HostEnvironment, child_environment};
use phrase_gym_coach::process::{Ending, Limits, ProcessError, ProcessSpec, run};
use tempfile::TempDir;

#[path = "support/executable.rs"]
mod executable;

const FAKE: &str = env!("CARGO_BIN_EXE_phrase-gym-fake-child");

struct Sandbox {
    root: TempDir,
    program: PathBuf,
    attempt: PathBuf,
}

// Places the fake CLI under a path with spaces and non-ASCII characters, as a Finder or Start-menu
// install location might have, and gives each run its own attempt directory.
fn sandbox() -> Sandbox {
    let root = tempfile::tempdir().unwrap();
    let tools = root.path().join("Tööls Ђ dir");
    let attempt = root.path().join("attempt Ünï Ђ");
    std::fs::create_dir_all(&tools).unwrap();
    std::fs::create_dir_all(&attempt).unwrap();
    let program = tools.join(format!("fake cli{}", std::env::consts::EXE_SUFFIX));
    executable::alias(Path::new(FAKE), &program);
    Sandbox {
        root,
        program,
        attempt,
    }
}

impl Sandbox {
    fn spec(&self, args: &[&str], limits: Limits) -> ProcessSpec {
        let mut host: Vec<(OsString, OsString)> = std::env::vars_os().collect();
        host.push(("PHRASE_GYM_TERMINAL_CANARY".into(), "1".into()));
        host.push(("ANTHROPIC_API_KEY".into(), "sk-ant-canary".into()));
        ProcessSpec {
            program: self.program.clone(),
            args: args.iter().map(OsString::from).collect(),
            cwd: self.attempt.clone(),
            env: child_environment(
                &HostEnvironment::from_variables(host),
                self.program.parent().unwrap(),
            ),
            stdout: self.attempt.join("stdout.log"),
            stderr: self.attempt.join("stderr.log"),
            limits,
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    fn text(&self, name: &str) -> String {
        std::fs::read_to_string(self.attempt.join(name)).unwrap_or_default()
    }
}

fn limits() -> Limits {
    Limits {
        output_bytes: 1024 * 1024,
        deadline: Duration::from_secs(30),
        grace: Duration::from_millis(500),
    }
}

async fn appears(path: &Path, needle: &str) {
    for _ in 0..1000 {
        if std::fs::read_to_string(path).is_ok_and(|text| text.contains(needle)) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("{} never contained {needle:?}", path.display());
}

async fn assert_stopped(heartbeat: &Path) {
    tokio::time::sleep(Duration::from_millis(300)).await;
    let before = std::fs::read_to_string(heartbeat).unwrap();
    tokio::time::sleep(Duration::from_millis(400)).await;
    let after = std::fs::read_to_string(heartbeat).unwrap();
    assert_eq!(before, after, "a descendant is still running");
}

async fn cancel_when(path: PathBuf, needle: &'static str) {
    appears(&path, needle).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_fixture_creation_and_launch_keeps_executables_immutable() {
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..8 {
        tasks.spawn(async {
            for _ in 0..8 {
                let sandbox = sandbox();
                #[cfg(unix)]
                assert!(
                    std::fs::symlink_metadata(&sandbox.program)
                        .unwrap()
                        .is_symlink()
                );
                let outcome = run(sandbox.spec(&["sleep", "0"], limits()), pending())
                    .await
                    .unwrap();
                assert_eq!(outcome.ending, Ending::Exited(Some(0)));
                assert!(outcome.stdout.closed);
                assert!(outcome.stderr.closed);
            }
        });
    }
    while let Some(result) = tasks.join_next().await {
        result.unwrap();
    }
}

#[tokio::test]
async fn passes_unicode_arguments_and_only_the_allowlisted_environment() {
    let sandbox = sandbox();
    let report = sandbox.path("report.json");
    let report_arg = report.to_str().unwrap();
    let args = [
        "report",
        report_arg,
        "Grüße",
        "Ђурђевак",
        "two words",
        "quote\"inside",
    ];
    let outcome = run(sandbox.spec(&args, limits()), pending()).await.unwrap();
    assert_eq!(outcome.ending, Ending::Exited(Some(0)));
    assert!(outcome.stdout.closed);
    assert_eq!(sandbox.text("stdout.log").trim(), "reported");
    let document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(
        document["args"],
        serde_json::json!(["Grüße", "Ђурђевак", "two words", "quote\"inside"])
    );
    let cwd = PathBuf::from(document["cwd"].as_str().unwrap());
    assert_eq!(
        cwd.canonicalize().unwrap(),
        sandbox.attempt.canonicalize().unwrap()
    );
    let environment = document["env"].as_object().unwrap();
    assert!(!environment.contains_key("PHRASE_GYM_TERMINAL_CANARY"));
    assert!(!environment.contains_key("ANTHROPIC_API_KEY"));
    let path = environment
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("PATH"))
        .map(|(_, value)| value.as_str().unwrap())
        .unwrap();
    assert!(path.starts_with(sandbox.program.parent().unwrap().to_str().unwrap()));
}

#[tokio::test]
async fn bounds_output_and_stops_a_flooding_child() {
    let sandbox = sandbox();
    let mut limits = limits();
    limits.output_bytes = 64 * 1024;
    let outcome = run(sandbox.spec(&["flood", "4000000"], limits), pending())
        .await
        .unwrap();
    assert_eq!(outcome.ending, Ending::OutputLimitExceeded);
    assert_eq!(outcome.stdout.bytes, 64 * 1024);
    assert!(outcome.stdout.truncated);
    let written = std::fs::metadata(sandbox.attempt.join("stdout.log")).unwrap();
    assert_eq!(written.len(), 64 * 1024);
}

#[tokio::test]
async fn times_out_a_hung_child() {
    let sandbox = sandbox();
    let mut limits = limits();
    limits.deadline = Duration::from_millis(300);
    let outcome = run(sandbox.spec(&["sleep", "60000"], limits), pending())
        .await
        .unwrap();
    assert_eq!(outcome.ending, Ending::TimedOut);
    assert!(outcome.elapsed < Duration::from_secs(15), "{outcome:?}");
}

#[tokio::test]
async fn cancellation_terminates_the_child_and_its_grandchildren() {
    let sandbox = sandbox();
    let heartbeat = sandbox.path("heartbeat Ђ.txt");
    let spec = sandbox.spec(
        &["grandchild", heartbeat.to_str().unwrap(), "wait"],
        limits(),
    );
    let stdout = spec.stdout.clone();
    let outcome = run(spec, cancel_when(stdout, "spawned")).await.unwrap();
    assert_eq!(outcome.ending, Ending::Cancelled);
    assert!(outcome.stdout.closed, "{outcome:?}");
    assert_stopped(&heartbeat).await;
}

#[tokio::test]
async fn a_normal_exit_still_kills_descendants_left_behind() {
    let sandbox = sandbox();
    let heartbeat = sandbox.path("orphan heartbeat.txt");
    let spec = sandbox.spec(
        &["grandchild", heartbeat.to_str().unwrap(), "exit"],
        limits(),
    );
    let outcome = run(spec, pending()).await.unwrap();
    assert_eq!(outcome.ending, Ending::Exited(Some(0)));
    assert!(outcome.elapsed < Duration::from_secs(15), "{outcome:?}");
    assert_stopped(&heartbeat).await;
}

#[tokio::test]
async fn dropping_the_supervisor_kills_the_whole_tree() {
    let sandbox = sandbox();
    let heartbeat = sandbox.path("dropped heartbeat.txt");
    let spec = sandbox.spec(
        &["grandchild", heartbeat.to_str().unwrap(), "wait"],
        limits(),
    );
    let stdout = spec.stdout.clone();
    tokio::select! {
        result = run(spec, pending()) => panic!("finished unexpectedly: {result:?}"),
        () = appears(&stdout, "spawned") => {}
    }
    assert_stopped(&heartbeat).await;
}

#[cfg(unix)]
#[tokio::test]
async fn escalates_to_kill_when_the_child_ignores_terminate() {
    let sandbox = sandbox();
    let spec = sandbox.spec(&["ignore-term", "60000"], limits());
    let stdout = spec.stdout.clone();
    let outcome = run(spec, cancel_when(stdout, "armed")).await.unwrap();
    assert_eq!(outcome.ending, Ending::Cancelled);
    assert!(outcome.elapsed >= Duration::from_millis(500), "{outcome:?}");
    assert!(outcome.elapsed < Duration::from_secs(15), "{outcome:?}");
}

#[tokio::test]
async fn reports_exit_codes_stderr_and_empty_output() {
    let sandbox = sandbox();
    let failed = run(
        sandbox.spec(&["exit", "3", "quota exceeded"], limits()),
        pending(),
    )
    .await
    .unwrap();
    assert_eq!(failed.ending, Ending::Exited(Some(3)));
    assert_eq!(failed.stdout.bytes, 0);
    assert!(sandbox.text("stderr.log").contains("quota exceeded"));
    let silent = run(sandbox.spec(&["sleep", "0"], limits()), pending())
        .await
        .unwrap();
    assert_eq!(silent.ending, Ending::Exited(Some(0)));
    assert_eq!((silent.stdout.bytes, silent.stdout.closed), (0, true));
}

#[tokio::test]
async fn a_missing_executable_is_a_spawn_error() {
    let sandbox = sandbox();
    let mut spec = sandbox.spec(&[], limits());
    spec.program = sandbox.path("missing cli");
    assert!(matches!(
        run(spec, pending()).await,
        Err(ProcessError::Spawn { .. })
    ));
}
