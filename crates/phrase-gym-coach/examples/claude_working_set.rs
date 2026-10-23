// Bounded real-Claude measurement: submits one frozen working set through the ordinary durable
// intake of a disposable Server with disposable Coach folders, prepares it once and reports.
// It spends real subscription usage, so it is never run by tests.
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use phrase_gym_coach::platform::{files, paths::CoachPaths};
use phrase_gym_coach::preparation::{
    self, attempt, readiness::ClaudeTool, selection::SelectedTool,
};
use phrase_gym_coach::transfer::{client::ServerClient, text, worker};

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let [server, directory, voice, fixture] = arguments.as_slice() else {
        return Err("Usage: claude_working_set LOOPBACK_SERVER FRESH_DIRECTORY PIPER_VOICE_JSON FIXTURE_JSON".into());
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
    let tool = ClaudeTool::check(&attempt::private_directory(&paths)?)?;
    let client = ServerClient::new(server)?;
    let accepted = text::submit_raw(&paths, &client.server, input, &client)?;
    let stop = Arc::new(AtomicBool::new(false));
    let started = Instant::now();
    let outcome = worker::tick(&client, &paths.state.join("worker.json"), |claim| {
        preparation::prepare_selected(&client, &paths, claim, &SelectedTool::claude(&tool), &stop)
    });
    println!(
        "{}",
        serde_json::json!({
            "claude": tool.version, "model": preparation::readiness::MODEL,
            "generation": accepted.generation_id, "elapsed_ms": started.elapsed().as_millis(),
            "attempts": paths.attempts(),
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
