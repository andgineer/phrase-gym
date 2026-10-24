// Bounded real-provider measurement: submits one frozen working set through the ordinary durable
// intake of a disposable Server with disposable Coach folders, prepares it once and reports.
// It spends real subscription usage, so it is never run by tests.
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use phrase_gym_coach::adapter::codex_readiness;
use phrase_gym_coach::platform::{files, paths::CoachPaths};
use phrase_gym_coach::preparation::{
    self, attempt, readiness::ClaudeTool, selection::SelectedTool,
};
use phrase_gym_coach::transfer::{client::ServerClient, text, worker};

const USAGE: &str = "Usage: working_set_measurement LOOPBACK_SERVER FRESH_DIRECTORY PIPER_VOICE_JSON FIXTURE_JSON [claude|codex]";

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let (server, directory, voice, fixture, tool) = match arguments.as_slice() {
        [server, directory, voice, fixture] => (server, directory, voice, fixture, "claude"),
        [server, directory, voice, fixture, tool] => {
            (server, directory, voice, fixture, tool.as_str())
        }
        _ => return Err(USAGE.into()),
    };
    let directory = PathBuf::from(directory);
    if !directory.is_absolute() || directory.exists() {
        return Err("Choose a fresh absolute directory outside any repository".into());
    }
    let paths = CoachPaths {
        config: directory.join("config"),
        cache: directory.join("cache"),
        logs: directory.join("logs"),
        state: directory.join("state"),
    };
    paths.ensure().map_err(|e| e.to_string())?;
    let voice: phrase_gym_coach::speech::PiperVoice = serde_json::from_slice(
        &files::read_selected(PathBuf::from(voice).as_path(), 16_384)?,
    )
    .map_err(|_| "Invalid Piper voice configuration")?;
    files::save(&paths.config.join("piper-voice.json"), &voice)?;
    let fixture: serde_json::Value = serde_json::from_slice(&files::read_selected(
        PathBuf::from(fixture).as_path(),
        65_536,
    )?)
    .map_err(|_| "Invalid fixture")?;
    let input = fixture["input"].as_str().ok_or("Fixture has no input")?;
    // Codex works in this repository with the normal Codex home; Claude in Coach's cache.
    let workspace = std::env::current_dir().map_err(|_| "Cannot resolve the repository")?;
    let (claude, codex) = match tool {
        "claude" => (
            Some(attempt::with_probe_directory(&paths, ClaudeTool::check)?),
            None,
        ),
        "codex" => (None, Some(codex_readiness::check()?)),
        _ => return Err(USAGE.into()),
    };
    let selected = match (&claude, &codex) {
        (Some(claude), _) => SelectedTool::claude(claude),
        (_, Some(codex)) => SelectedTool::codex(codex, &workspace),
        _ => unreachable!("one tool was checked"),
    };
    let client = ServerClient::new(server)?;
    let accepted = text::submit_raw(&paths, &client.server, input, &client)?;
    let stop = Arc::new(AtomicBool::new(false));
    let started = Instant::now();
    let outcome = worker::tick(&client, &paths.state.join("worker.json"), |claim| {
        preparation::prepare_selected(&client, &paths, claim, &selected, &stop)
    });
    println!(
        "{}",
        serde_json::json!({
            "tool": selected.identity(),
            "generation": accepted.generation_id, "elapsed_ms": started.elapsed().as_millis(),
            "attempts": selected.attempts(&paths),
            "outcome": match &outcome { Ok(receipt) => serde_json::json!({"published": receipt}), Err(problem) => serde_json::json!({"failed": problem}) },
        })
    );
    outcome.map(|_| ())
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
