use std::ffi::OsString;
use std::future::pending;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;

use crate::adapter::CliAdapter;
use crate::adapter::environment::{HostEnvironment, child_environment};
use crate::platform::paths::CoachPaths;
use crate::platform::{resources, tools};
use crate::process::{Ending, Limits, ProcessSpec, run};

const ARGUMENTS: [&str; 3] = ["Grüße", "Ђурђевак", "two words"];

const EXPECTED: [&str; 5] = [
    "arguments_round_trip",
    "working_directory",
    "environment_isolated",
    "output_bounded",
    "descendants_cleaned",
];

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub check: &'static str,
    pub passed: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct ProcessChecks {
    pub fake_child: Option<PathBuf>,
    pub attempt_directory: Option<PathBuf>,
    pub outcomes: Vec<Outcome>,
    pub tools: Vec<ToolProbe>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ToolProbe {
    pub adapter: CliAdapter,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
}

impl ProcessChecks {
    pub fn passed(&self) -> bool {
        self.errors.is_empty()
            && EXPECTED.iter().all(|expected| {
                self.outcomes
                    .iter()
                    .any(|outcome| outcome.check == *expected && outcome.passed)
            })
    }

    fn record(&mut self, check: &'static str, passed: bool) {
        self.outcomes.push(Outcome { check, passed });
    }
}

// The packaged app's own copy of the supervision proof: the bundled fake CLI runs from the
// installed location with Coach's allowlisted environment and per-user attempt folder.
pub fn run_all(executable: &Path) -> ProcessChecks {
    let mut checks = ProcessChecks::default();
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            checks.errors.push(format!("runtime: {error}"));
            return checks;
        }
    };
    runtime.block_on(async {
        if let Err(problem) = fake_child_checks(executable, &mut checks).await {
            checks.errors.push(problem);
        }
        if let Some(attempt) = checks.attempt_directory.clone() {
            checks.tools = tool_probes(&attempt).await;
        }
    });
    checks
}

async fn fake_child_checks(executable: &Path, checks: &mut ProcessChecks) -> Result<(), String> {
    let name = resources::executable_name("phrase-gym-fake-child");
    let fake = resources::locate(executable, &name).ok_or(format!("{name} is not bundled"))?;
    let paths = CoachPaths::for_current_user().map_err(|error| error.to_string())?;
    let attempt = paths.attempts().join("smoke attempt Ünï Ђ");
    std::fs::create_dir_all(&attempt).map_err(|error| error.to_string())?;
    checks.fake_child = Some(fake.clone());
    checks.attempt_directory = Some(attempt.clone());
    let spec = |args: Vec<OsString>, limits: Limits| ProcessSpec {
        program: fake.clone(),
        args,
        cwd: attempt.clone(),
        env: child_environment(&HostEnvironment::current(), fake.parent().unwrap_or(&fake)),
        stdout: attempt.join("stdout.log"),
        stderr: attempt.join("stderr.log"),
        limits,
    };
    let limits = Limits {
        output_bytes: 64 * 1024,
        deadline: Duration::from_secs(30),
        grace: Duration::from_millis(500),
    };

    let report = attempt.join("report.json");
    let mut args = vec![OsString::from("report"), report.clone().into_os_string()];
    args.extend(ARGUMENTS.iter().map(OsString::from));
    run(spec(args, limits), pending())
        .await
        .map_err(|error| error.to_string())?;
    let document: serde_json::Value = std::fs::read_to_string(&report)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .ok_or("the fake CLI wrote no report")?;
    checks.record(
        "arguments_round_trip",
        document["args"] == serde_json::json!(ARGUMENTS),
    );
    let cwd = document["cwd"]
        .as_str()
        .and_then(|cwd| Path::new(cwd).canonicalize().ok());
    checks.record("working_directory", cwd == attempt.canonicalize().ok());
    let isolated = document["env"].as_object().is_some_and(|environment| {
        [
            "PHRASE_GYM_TERMINAL_CANARY",
            "ANTHROPIC_API_KEY",
            "OPENAI_API_KEY",
        ]
        .iter()
        .all(|name| !environment.contains_key(*name))
    });
    checks.record("environment_isolated", isolated);

    let flood = vec![OsString::from("flood"), OsString::from("2000000")];
    let flooded = run(spec(flood, limits), pending())
        .await
        .map_err(|error| error.to_string())?;
    checks.record(
        "output_bounded",
        flooded.ending == Ending::OutputLimitExceeded
            && flooded.stdout.bytes == limits.output_bytes,
    );

    let heartbeat = attempt.join("heartbeat.txt");
    let _ = std::fs::remove_file(&heartbeat);
    let tree = vec![
        OsString::from("grandchild"),
        heartbeat.clone().into_os_string(),
        OsString::from("exit"),
    ];
    run(spec(tree, limits), pending())
        .await
        .map_err(|error| error.to_string())?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let before = std::fs::read_to_string(&heartbeat).ok();
    tokio::time::sleep(Duration::from_millis(400)).await;
    let after = std::fs::read_to_string(&heartbeat).ok();
    checks.record("descendants_cleaned", before.is_some() && before == after);
    Ok(())
}

// `--version` costs no model usage; it proves the real CLI starts from a GUI launch without the
// shell's PATH and with the stripped environment.
async fn tool_probes(directory: &Path) -> Vec<ToolProbe> {
    let host = HostEnvironment::current();
    let mut probes = Vec::new();
    for adapter in CliAdapter::ALL {
        let path = tools::discover(adapter, &host);
        let mut version = None;
        if let Some(executable) = &path {
            let spec = ProcessSpec {
                program: executable.clone(),
                args: vec![OsString::from("--version")],
                cwd: directory.to_path_buf(),
                env: child_environment(&host, executable.parent().unwrap_or(executable)),
                stdout: directory.join(format!("{}-version.out", adapter.executable_name())),
                stderr: directory.join(format!("{}-version.err", adapter.executable_name())),
                limits: Limits {
                    output_bytes: 64 * 1024,
                    deadline: Duration::from_secs(30),
                    grace: Duration::from_millis(500),
                },
            };
            let stdout = spec.stdout.clone();
            if let Ok(report) = run(spec, pending()).await
                && report.ending == Ending::Exited(Some(0))
            {
                version = std::fs::read_to_string(stdout)
                    .ok()
                    .and_then(|text| text.lines().next().map(str::trim).map(str::to_owned));
            }
        }
        probes.push(ToolProbe {
            adapter,
            path,
            version,
        });
    }
    probes
}
