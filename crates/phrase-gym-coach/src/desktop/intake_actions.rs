use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use super::intake::{Command, Event};
use crate::platform::paths::CoachPaths;
use crate::transfer::{client::ServerClient, pending, runner};

pub(super) fn handle(
    command: Command,
    server: &str,
    client: Option<&ServerClient>,
    paths: &CoachPaths,
    emit: &dyn Fn(Event),
    stop: &Arc<AtomicBool>,
) -> Result<(), String> {
    let command = match command {
        Command::DropText(path) => {
            let text = super::dropped_text::read(&path).map_err(String::from)?;
            emit(Event::Draft(text.clone()));
            Command::Text(text)
        }
        command => command,
    };
    if let Command::Text(phrase) = &command {
        crate::transfer::text::TextDraft::retain(
            &paths.state.join("pending-text.json"),
            server,
            phrase,
        )?;
        let client =
            client.ok_or("Text retained locally; restore the Server connection to send it")?;
        crate::transfer::text::submit_raw(paths, server, phrase, client)?;
        emit(Event::Status(
            "Text accepted durably by Server; preparation continues automatically".into(),
        ));
        return Ok(());
    }
    match command {
        Command::RetryPreparation => retry_preparation(client, paths, emit),
        command => {
            let path = paths.state.join("pending-send.json");
            let mut retained = match (pending::load(&path)?, &command) {
                (Some(retained), Command::Retry) => retained,
                (Some(_), _) => return Err("A saved input is still pending. Restore its connection and retry before adding another lesson".into()),
                (None, Command::Retry) => return Ok(()),
                _ => return Err("Unsupported intake command".into()),
            };
            pending::save(&path, &retained)?;
            let client = client.ok_or("Input saved locally, acceptance unconfirmed. Restore the Server connection; delivery retries automatically")?;
            let mut prepared = None;
            let receipt = runner::deliver_with(
                client,
                &path,
                &mut retained,
                |s| emit(Event::Status(s.into())),
                |claim| {
                    let output =
                        if crate::supplied::SuppliedLesson::parse(&claim.input.context).is_ok() {
                            crate::supplied::output(claim)
                        } else {
                            emit(Event::Status(
                                "Accepted durably; preparing with the selected CLI…".into(),
                            ));
                            super::selected::prepare(client, paths, claim, stop)
                        }?;
                    prepared = Some(output.clone());
                    Ok(output)
                },
            )?;
            published(client, &receipt, prepared.as_ref(), emit);
            Ok(())
        }
    }
}

pub(super) fn published(
    client: &ServerClient,
    receipt: &phrase_gym_contracts::inbox::PublicationReceipt,
    output: Option<&phrase_gym_contracts::inbox::LessonOutput>,
    emit: &dyn Fn(Event),
) {
    if !receipt.target_ids.is_empty() {
        emit(Event::Published(format!(
            "{}/learn/en/lessons/{}",
            client.server, receipt.lesson_id
        )));
    }
    emit(Event::Status(super::text::confirmed_additions(
        output,
        receipt.target_ids.len(),
    )));
}

fn retry_preparation(
    client: Option<&ServerClient>,
    paths: &CoachPaths,
    emit: &dyn Fn(Event),
) -> Result<(), String> {
    let client = client.ok_or("Restore the Server connection before retrying preparation")?;
    let path = paths.state.join("pending-send.json");
    if path.exists() {
        crate::transfer::retry::preparation(client, &path)?;
    } else {
        crate::transfer::text::retry_preparation(paths, client, None)?;
    }
    emit(Event::Status(
        "Preparation retry accepted; automatic processing resumes".into(),
    ));
    Ok(())
}
