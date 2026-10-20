use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use super::intake::{Command, Event, IntakeState};
use crate::platform::paths::CoachPaths;
use crate::preparation::{self, readiness::ClaudeTool};
use crate::transfer::{client::ServerClient, material::Material, pending, runner};

pub(super) fn handle(
    command: Command,
    server: &str,
    client: Option<&ServerClient>,
    paths: &CoachPaths,
    emit: &dyn Fn(Event),
    stop: &Arc<AtomicBool>,
) -> Result<(), String> {
    match command {
        Command::PreviewAudio(path) => {
            super::audio_preview::play(&PathBuf::from(path), paths, stop)
        }
        Command::RetryPreparation => retry_preparation(client, paths, emit),
        Command::ChooseLesson => {
            let file = rfd::FileDialog::new()
                .add_filter("Prepared lesson", &["json"])
                .pick_file();
            if let Some(file) = file {
                load(&file, emit)?;
            }
            Ok(())
        }
        Command::Load(file) => load(&PathBuf::from(file), emit),
        Command::Demo => {
            emit(Event::Draft(
                include_str!("../../../../fixtures/coach/supplied-lesson.json").into(),
            ));
            emit(Event::Status("Supplied demo text loaded and validated locally. Choose its matching original audio to publish automatically".into()));
            Ok(())
        }
        Command::ChooseAudio => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Original WAV audio", &["wav"])
                .pick_file()
            {
                emit(Event::Audio(path.to_string_lossy().into_owned()));
            }
            Ok(())
        }
        command => {
            let path = paths.state.join("pending-send.json");
            let mut retained = match (pending::load(&path)?, &command) {
                (Some(retained), Command::Retry) => retained,
                (Some(_), _) => return Err("A saved input is still pending. Restore its connection and retry before adding another lesson".into()),
                (None, Command::Send { json, audio }) => runner::create(server, json, &PathBuf::from(audio))?,
                (None, Command::Raw { audio, .. }) => {
                    let raw = IntakeState::raw(&command).ok_or("Invalid raw input")?;
                    raw.validate()?;
                    runner::create_material(server, Material::Raw(raw), &PathBuf::from(audio))?
                }
                (None, Command::Retry) => return Ok(()),
                _ => return Err("Unsupported intake command".into()),
            };
            pending::save(&path, &retained)?;
            let client = client.ok_or("Input saved locally, acceptance unconfirmed. Restore the Server connection; delivery retries automatically")?;
            let receipt = runner::deliver_with(
                client,
                &path,
                &mut retained,
                |s| emit(Event::Status(s.into())),
                |claim| {
                    if crate::supplied::SuppliedLesson::parse(&claim.input.context).is_ok() {
                        return crate::supplied::output(claim);
                    }
                    emit(Event::Status(
                        "Accepted durably; preparing with the selected Claude CLI…".into(),
                    ));
                    let check = preparation::attempt::private_directory(paths)?;
                    let tool = ClaudeTool::check(&check)?;
                    emit(Event::SelectedClaude(tool.executable.clone()));
                    preparation::prepare(client, paths, claim, &tool, stop)
                },
            )?;
            published(client, &receipt, emit);
            Ok(())
        }
    }
}

fn load(path: &std::path::Path, emit: &dyn Fn(Event)) -> Result<(), String> {
    emit(Event::Status(
        "Loading and validating the selected prepared lesson…".into(),
    ));
    let bytes = crate::platform::files::read_selected(path, 8000)?;
    let json = String::from_utf8(bytes).map_err(|_| "Prepared lesson must be UTF-8 JSON")?;
    crate::supplied::SuppliedLesson::parse(&json)?;
    emit(Event::Draft(json));
    emit(Event::Status("Prepared lesson loaded and validated locally; publication is not yet confirmed. Choose the matching original audio".into()));
    Ok(())
}

pub(super) fn published(
    client: &ServerClient,
    receipt: &phrase_gym_contracts::inbox::PublicationReceipt,
    emit: &dyn Fn(Event),
) {
    if let Some(id) = receipt.target_ids.first() {
        emit(Event::Published(format!(
            "{}/learn/en/phrases/{id}",
            client.server
        )));
    }
    emit(Event::Status(
        "Published and confirmed by Server. Open browser practice and choose Start session".into(),
    ));
}

fn retry_preparation(
    client: Option<&ServerClient>,
    paths: &CoachPaths,
    emit: &dyn Fn(Event),
) -> Result<(), String> {
    let client = client.ok_or("Restore the Server connection before retrying preparation")?;
    let path = paths.state.join("pending-send.json");
    let mut retained =
        pending::load(&path)?.ok_or("Retry the accepted lesson from its browser status panel")?;
    if retained.server != client.server || retained.identity != Some(client.identity) {
        return Err(
            "Restore the original pending input account before retrying preparation".into(),
        );
    }
    let input = retained
        .submission
        .as_ref()
        .ok_or("Delivery is not yet accepted; restore the connection first")?;
    let receipt: Option<phrase_gym_contracts::inbox::LessonReceipt> = client.get(&format!(
        "/api/v1/inbox/submissions/{}",
        input.submission_id
    ))?;
    let receipt =
        receipt.ok_or("Server has not confirmed this submission; restore delivery first")?;
    client.post::<_, phrase_gym_contracts::inbox::LessonStatus>(
        &format!("/api/v1/inbox/lessons/{}/retry", receipt.generation_id),
        &(),
        false,
    )?;
    retained.claim_id = uuid::Uuid::new_v4();
    retained.output = None;
    retained.failure = None;
    pending::save(&path, &retained)?;
    emit(Event::Status(
        "Preparation retry accepted; automatic processing resumes".into(),
    ));
    Ok(())
}
