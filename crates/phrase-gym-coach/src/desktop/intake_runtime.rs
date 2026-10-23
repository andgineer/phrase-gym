use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use super::intake_actions::{handle, published};
use super::{
    connection,
    intake::{Command, Event},
};
use crate::platform::{paths::CoachPaths, settings::Settings};
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
    let Some(paths) = initialize(emit) else {
        return;
    };
    let mut server = match Settings::load(&paths.config.join("settings.json")) {
        Ok(settings) => settings.map_or_else(
            phrase_gym_contracts::connection::default_server_origin,
            |s| s.server,
        ),
        Err(problem) => {
            emit(Event::Status(problem));
            return;
        }
    };
    emit(Event::Configured(server.clone()));
    let mut client = None;
    let mut last_summary = None;
    let mut last = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
    while !stop.load(Ordering::SeqCst) {
        let command = match receive.recv_timeout(Duration::from_millis(100)) {
            Ok(command) => Some(command),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        if let Some(command) = command {
            let result = match command {
                Command::Connect { server: next } => {
                    client = None;
                    server = next;
                    connect(&server, &paths, &mut client, emit)
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
            if let Err(problem) = connect(&server, &paths, &mut client, emit) {
                emit(Event::Status(problem));
            }
            continue;
        }
        let Some(client) = &client else {
            continue;
        };
        summary(client, &mut last_summary, emit);
        let draft_path = paths.state.join("pending-text.json");
        if draft_path.exists() {
            let retained =
                crate::platform::files::read_selected(&draft_path, 8192).and_then(|bytes| {
                    serde_json::from_slice::<crate::transfer::text::TextDraft>(&bytes)
                        .map_err(|_| "Retained text is unreadable".into())
                });
            match retained.and_then(|draft| {
                crate::transfer::text::submit_raw(&paths, &server, &draft.phrase, client)
            }) {
                Ok(_) => emit(Event::Status(
                    "Original text accepted durably by Server".into(),
                )),
                Err(problem) => {
                    emit(Event::Status(problem));
                    continue;
                }
            }
        }
        let pending_path = paths.state.join("pending-send.json");
        if pending_path.exists() {
            if let Err(problem) = handle(Command::Retry, &server, Some(client), &paths, emit, stop)
            {
                emit(Event::Status(problem));
            }
            continue;
        }
        prepare_waiting(client, &paths, emit, stop);
    }
}

fn prepare_waiting(
    client: &ServerClient,
    paths: &CoachPaths,
    emit: &dyn Fn(Event),
    stop: &Arc<AtomicBool>,
) {
    let mut prepared = None;
    let result = worker::tick(client, &paths.state.join("worker.json"), |claim| {
        let output = if crate::supplied::SuppliedLesson::parse(&claim.input.context).is_ok() {
            crate::supplied::output(claim)
        } else {
            emit(Event::Status(
                "Accepted by Server; preparing your English working set…".into(),
            ));
            super::selected::prepare(client, paths, claim, stop)
        }?;
        prepared = Some(output.clone());
        Ok(output)
    });
    match result {
        Ok(Some(receipt)) => published(client, &receipt, prepared.as_ref(), emit),
        Ok(None) => {}
        Err(problem) => emit(Event::Status(problem)),
    }
}

fn connect(
    server: &str,
    paths: &CoachPaths,
    client: &mut Option<ServerClient>,
    emit: &dyn Fn(Event),
) -> Result<(), String> {
    let connected = connection::connect(server, paths)?;
    emit(Event::Connected(connected.server.clone()));
    emit(Event::Status(String::new()));
    *client = Some(connected);
    Ok(())
}

// Unchanged counts wake nobody: an idle window repaints only when something it shows changes.
fn summary(client: &ServerClient, last: &mut Option<String>, emit: &dyn Fn(Event)) {
    let text = match client
        .get::<phrase_gym_contracts::readiness::CoachSummary>("/api/v1/coach/summary")
    {
        Ok(summary) => format!(
            "{} lessons · {} phrases · {} ready · {} awaiting audio",
            summary.lessons, summary.targets, summary.ready, summary.awaiting_audio
        ),
        Err(_) => "Library counts unavailable · last update is stale".into(),
    };
    if last.as_ref() != Some(&text) {
        *last = Some(text.clone());
        emit(Event::Summary(text));
    }
}

fn initialize(emit: &dyn Fn(Event)) -> Option<CoachPaths> {
    let Ok(paths) = CoachPaths::for_current_user().and_then(|paths| {
        paths.ensure()?;
        Ok(paths)
    }) else {
        emit(Event::Status(
            "Cannot initialize protected Coach folders; restore disk access and restart".into(),
        ));
        return None;
    };
    Some(paths)
}
