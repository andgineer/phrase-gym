use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use super::intake::Event;
use crate::adapter::environment::HostEnvironment;
use crate::platform::{paths::CoachPaths, resources};
use crate::preparation::{self, readiness::ClaudeTool};
use crate::transfer::{client::ServerClient, worker};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmokeInput {
    pub server: String,
    pub directory: PathBuf,
}

pub fn run(input: &SmokeInput, emit: &dyn Fn(Event), stop: &Arc<AtomicBool>) -> Result<(), String> {
    if !input.directory.is_absolute() || input.directory.exists() {
        return Err("Smoke intake needs a fresh absolute disposable directory".into());
    }
    let paths = CoachPaths {
        config: input.directory.join("config"),
        cache: input.directory.join("cache"),
        logs: input.directory.join("logs"),
        state: input.directory.join("state"),
    };
    paths.ensure().map_err(|e| e.to_string())?;
    let executable = std::env::current_exe().map_err(|_| "Cannot find packaged executable")?;
    let fake = resources::resource_directory(&executable).join(format!(
        "phrase-gym-fake-child{}",
        std::env::consts::EXE_SUFFIX
    ));
    let tool = ClaudeTool {
        executable: fake,
        host: HostEnvironment::default(),
        version: "2.1.309".into(),
    };
    let client = ServerClient::new(&input.server)?;
    emit(Event::Connected(client.server.clone()));
    let receipt = worker::tick(&client, &paths.state.join("worker.json"), |claim| {
        preparation::prepare(&client, &paths, claim, &tool, stop)
    })?
    .ok_or("No accepted synthetic browser input for the packaged worker")?;
    crate::platform::files::save(&paths.state.join("publication.json"), &receipt)?;
    if let Some(id) = receipt.target_ids.first() {
        emit(Event::Published(format!(
            "{}/learn/en/phrases/{id}",
            client.server
        )));
    }
    emit(Event::Status(
        "Packaged synthetic raw intake published; no provider was called".into(),
    ));
    Ok(())
}
