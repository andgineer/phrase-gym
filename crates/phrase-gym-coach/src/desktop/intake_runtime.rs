use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use super::intake_actions::{handle, published};
use super::{
    connection,
    intake::{Command, Event},
    settings::Settings,
};
use crate::platform::paths::CoachPaths;
use crate::preparation::{self, readiness::ClaudeTool};
use crate::transfer::{client::ServerClient, worker};

pub fn run(
    receive: &Receiver<Command>,
    emit: &dyn Fn(Event),
    stop: &Arc<AtomicBool>,
    processing: bool,
) {
    if !processing {
        return;
    }
    let Ok(paths) = CoachPaths::for_current_user().and_then(|paths| {
        paths.ensure()?;
        Ok(paths)
    }) else {
        emit(Event::Status(
            "Cannot initialize protected Coach folders; restore disk access and restart".into(),
        ));
        return;
    };
    let mut server = match Settings::load(&paths.config.join("settings.json")) {
        Ok(settings) => settings.map_or_else(|| "http://127.0.0.1:8000".into(), |s| s.server),
        Err(problem) => {
            emit(Event::Status(problem));
            return;
        }
    };
    let mut client = None;
    let mut last = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
    let mut worker_problem = None;
    while !stop.load(Ordering::SeqCst) {
        let command = match receive.recv_timeout(Duration::from_millis(100)) {
            Ok(command) => Some(command),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        if let Some(command) = command {
            let result = match command {
                Command::Connect {
                    server: next,
                    submit,
                    worker,
                } => {
                    client = None;
                    server = next;
                    worker_problem = None;
                    connect(&server, &submit, &worker, &paths, &mut client, emit)
                }
                command => handle(command, &server, client.as_ref(), &paths, emit, stop),
            };
            emit(Event::Finished(result));
        }
        if last.elapsed() < Duration::from_secs(5) {
            continue;
        }
        last = Instant::now();
        if client.is_none() {
            if let Err(problem) = connect(&server, "", "", &paths, &mut client, emit) {
                emit(Event::Status(problem));
            }
            continue;
        }
        let Some(client) = &client else {
            continue;
        };
        let pending_path = paths.state.join("pending-send.json");
        if pending_path.exists() {
            if let Err(problem) = handle(Command::Retry, &server, Some(client), &paths, emit, stop)
            {
                emit(Event::Status(problem));
            }
            continue;
        }
        if worker_problem.is_some() {
            continue;
        }
        let result = worker::tick(client, &paths.state.join("worker.json"), |claim| {
            if crate::supplied::SuppliedLesson::parse(&claim.input.context).is_ok() {
                return crate::supplied::output(claim);
            }
            emit(Event::Status(
                "Accepted by Server; preparing one English phrase with Claude…".into(),
            ));
            let check = preparation::attempt::private_directory(&paths)?;
            let tool = ClaudeTool::check(&check)?;
            emit(Event::SelectedClaude(tool.executable.clone()));
            preparation::prepare(client, &paths, claim, &tool, stop)
        });
        match result {
            Ok(Some(receipt)) => published(client, &receipt, emit),
            Ok(None) => {}
            Err(problem) => {
                if !problem.starts_with("Server unavailable")
                    && !problem.contains("response")
                    && !problem.starts_with("Expired preparation reconciled")
                {
                    worker_problem = Some(problem.clone());
                }
                emit(Event::Status(problem));
            }
        }
    }
}

fn connect(
    server: &str,
    submit: &str,
    worker: &str,
    paths: &CoachPaths,
    client: &mut Option<ServerClient>,
    emit: &dyn Fn(Event),
) -> Result<(), String> {
    let connected = connection::connect(server, submit, worker, paths)?;
    emit(Event::Connected(connected.server.clone()));
    emit(Event::Status(
        "Connected automatically; ready for supplied or raw English input with original WAV audio"
            .into(),
    ));
    *client = Some(connected);
    Ok(())
}
