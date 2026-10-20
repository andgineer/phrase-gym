// Browser integration uses this native stand-in only; it never discovers or starts a provider CLI.
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use phrase_gym_coach::adapter::environment::HostEnvironment;
use phrase_gym_coach::platform::paths::CoachPaths;
use phrase_gym_coach::preparation::{self, readiness::ClaudeTool};
use phrase_gym_coach::transfer::{client::ServerClient, worker};

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let [server, directory] = arguments.as_slice() else {
        return Err("Usage: fake_worker LOOPBACK_SERVER FRESH_TEMP_DIRECTORY".into());
    };
    let directory = PathBuf::from(directory);
    let paths = CoachPaths {
        config: directory.join("config"),
        cache: directory.join("cache"),
        logs: directory.join("logs"),
        state: directory.join("state"),
    };
    paths.ensure().map_err(|e| e.to_string())?;
    let executable = std::env::current_exe().map_err(|_| "Cannot find native test executable")?;
    let fake = executable
        .parent()
        .and_then(|path| path.parent())
        .ok_or("Cannot find test binaries")?
        .join(format!(
            "phrase-gym-fake-child{}",
            std::env::consts::EXE_SUFFIX
        ));
    let tool = ClaudeTool {
        executable: fake,
        host: HostEnvironment::default(),
    };
    let token = "synthetic-browser-00000000000000000000";
    let client = ServerClient::new(server, token.into(), token.into())?;
    let receipt = worker::tick(&client, &paths.state.join("worker.json"), |claim| {
        preparation::prepare(
            &client,
            &paths,
            claim,
            &tool,
            &Arc::new(AtomicBool::new(false)),
        )
    })?
    .ok_or("No accepted synthetic lesson to prepare")?;
    println!(
        "{}",
        serde_json::to_string(&receipt).map_err(|_| "Cannot encode synthetic receipt")?
    );
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(problem) => {
            eprintln!("{problem}");
            std::process::ExitCode::FAILURE
        }
    }
}
