#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use phrase_gym_coach::adapter::invocation::{AttemptFiles, Effort, LaunchContext, ModelSelection};
use phrase_gym_coach::adapter::{CliAdapter, codex_readiness};
use serde_json::json;

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [destination] = args.as_slice() else {
        return Err(
            "Usage: codex_preflight ABSOLUTE_FRESH_DIRECTORY (never starts a model)".into(),
        );
    };
    let destination = PathBuf::from(destination);
    if !destination.is_absolute() || destination.exists() {
        return Err("Choose a fresh absolute directory outside repositories".into());
    }
    let parent = destination
        .parent()
        .ok_or("Destination has no parent")?
        .canonicalize()
        .map_err(|_| "Destination parent must exist")?;
    if parent.ancestors().any(|p| p.join(".git").exists()) {
        return Err("Measurement staging must be outside repositories".into());
    }
    let ready = codex_readiness::check()?;
    std::fs::create_dir(&destination).map_err(|_| "Cannot create staging directory")?;
    #[cfg(unix)]
    {
        std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| "Cannot restrict staging directory")?;
    }
    let directory = destination
        .canonicalize()
        .map_err(|_| "Cannot resolve staging directory")?;
    let attempt = AttemptFiles::new(directory, "prompt.md").map_err(|e| e.to_string())?;
    std::fs::write(
        attempt.directory().join("input.txt"),
        "Zdravo — Здраво — Grüß Gott 7319\n",
    )
    .map_err(|_| "Cannot stage synthetic input")?;
    std::fs::write(attempt.prompt(), "Read input.txt in the current directory. Reply with exactly its first line and nothing else.")
        .map_err(|_| "Cannot stage synthetic prompt")?;
    let model = ModelSelection::new(codex_readiness::MODEL, Some(Effort::Low))
        .map_err(|e| e.to_string())?;
    let invocation = CliAdapter::Codex
        .invocation(&LaunchContext {
            executable: &ready.executable,
            attempt: &attempt,
            model: &model,
            host: &ready.host,
            codex_home: &ready.home,
        })
        .map_err(|e| e.to_string())?;
    let manifest = json!({
        "model_started": false, "own_home_isolation_accepted": false,
        "version": codex_readiness::VERIFIED_VERSION, "model": codex_readiness::MODEL,
        "program": invocation.program, "arguments": invocation.arguments(),
        "cwd": invocation.cwd, "codex_home": ready.home,
        "environment_names": invocation.env.iter().map(|(k, _)| k.to_string_lossy()).collect::<Vec<_>>(),
        "proposed_limits": {"starts": 1, "deadline_seconds": 90, "output_bytes_per_stream": 65536, "stdin": "null"}
    });
    std::fs::write(
        destination.join("measurement.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|_| "Cannot serialize manifest")?,
    )
    .map_err(|_| "Cannot stage manifest")?;
    println!(
        "Coach-owned Codex setup ready; no model started. Inspect measurement.json. Real execution requires explicit run-budget authorization; own-home isolation remains unverified."
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
