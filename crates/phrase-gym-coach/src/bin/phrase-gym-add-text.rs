use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use phrase_gym_coach::adapter::codex_readiness;
use phrase_gym_coach::platform::{instance::InstanceGuard, paths::CoachPaths};
use phrase_gym_coach::preparation::{self, selection::SelectedTool};
use phrase_gym_coach::transfer::{client::ServerClient, text, worker};

fn run() -> Result<(), String> {
    let mut arguments = std::env::args().skip(1);
    let first = arguments.next().ok_or("Usage: phrase-gym-add-text [--retry-preparation] 'One exact English phrase.' [http://127.0.0.1:8000]")?;
    let retry = first == "--retry-preparation";
    let phrase = if retry { arguments.next() } else { Some(first) }
        .ok_or("Usage: phrase-gym-add-text 'One exact English phrase.' [http://127.0.0.1:8000]")?;
    let server = arguments
        .next()
        .unwrap_or_else(phrase_gym_contracts::connection::default_server_origin);
    if arguments.next().is_some() {
        return Err("Expected one phrase and optional local Server URL".into());
    }
    let paths = CoachPaths::for_current_user().map_err(|e| e.to_string())?;
    paths.ensure().map_err(|e| e.to_string())?;
    let _guard = InstanceGuard::acquire(&paths.instance_lock()).map_err(|e| e.to_string())?;
    let workspace = preparation::workspace::save(
        &paths,
        &std::env::current_dir().map_err(|_| "Cannot resolve repository workspace")?,
    )?;
    let stop = Arc::new(AtomicBool::new(false));
    // Check subscription and Server before starting either provider. Neither check creates a job.
    let tool = codex_readiness::check()?;
    let client = ServerClient::new(&server)?;
    let _: phrase_gym_contracts::profiles::ProfileRevision = client.get("/api/v1/profile")?;
    if retry {
        text::retry_preparation(&paths, &client, Some(&phrase))?;
    } else {
        let accepted = text::submit_raw(&paths, &client.server, &phrase, &client)?;
        println!("Original text accepted durably: {}", accepted.generation_id);
    }
    let receipt = worker::tick(&client, &paths.state.join("worker.json"), |claim| {
        preparation::prepare_selected(
            &client,
            &paths,
            claim,
            &SelectedTool::codex(&tool, &workspace),
            &stop,
        )
    })?
    .ok_or("Text accepted; keep Coach running to prepare waiting work")?;
    phrase_gym_coach::platform::settings::Settings::save_selected(
        &paths.config.join("settings.json"),
        &client.server,
        phrase_gym_coach::adapter::CliAdapter::Codex,
    )?;
    println!(
        "Published {} ready target(s). Open {}/learn/en and choose Start session.",
        receipt.target_ids.len(),
        client.server
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::FAILURE
        }
    }
}
