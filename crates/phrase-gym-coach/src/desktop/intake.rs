use std::path::PathBuf;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use crate::platform::credentials::CredentialStore;
use crate::platform::paths::CoachPaths;
use crate::transfer::{
    client::{self, ServerClient},
    pending, runner,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Connect {
        server: String,
        submit: String,
        worker: String,
    },
    Send {
        json: String,
        audio: String,
    },
    Retry,
    Load(String),
}

pub enum Event {
    Status(String),
    Connected(String),
    Draft(String),
    Published(String),
    Finished(Result<(), String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntakeState {
    pub server: String,
    pub submit: String,
    pub worker: String,
    pub audio: String,
    pub lesson_file: String,
    pub status: String,
    pub connected: bool,
    pub busy: bool,
    pub practice: Option<String>,
    pub command: Option<Command>,
}

impl Default for IntakeState {
    fn default() -> Self {
        Self {
            server: "http://127.0.0.1:8000".into(),
            submit: String::new(),
            worker: String::new(),
            audio: String::new(),
            lesson_file: String::new(),
            status: "Connect to the local Server to send a supplied prepared lesson".into(),
            connected: false,
            busy: false,
            practice: None,
            command: None,
        }
    }
}

pub struct IntakeRunner {
    pub commands: SyncSender<Command>,
    pub events: Receiver<Event>,
}

impl IntakeRunner {
    pub fn spawn(wake: impl Fn() + Send + 'static) -> Self {
        let (commands, receive) = sync_channel(2);
        let (send, events) = sync_channel(8);
        std::thread::spawn(move || {
            let emit = |event| {
                let _ = send.send(event);
                wake();
            };
            let mut client = None;
            for command in receive {
                let result = handle(command, &mut client, &emit);
                emit(Event::Finished(result));
            }
        });
        Self { commands, events }
    }
}

fn handle(
    command: Command,
    client: &mut Option<ServerClient>,
    emit: &dyn Fn(Event),
) -> Result<(), String> {
    match command {
        Command::Connect {
            server,
            submit,
            worker,
        } => {
            *client = None;
            let server = client::loopback(&server)?;
            let store = CredentialStore::native().map_err(|e| e.to_string())?;
            let credential = |role: &str, supplied: &str| -> Result<String, String> {
                let account = format!("{role}:{server}");
                if !supplied.is_empty() {
                    store.write(&account, supplied).map_err(|e| e.to_string())?;
                }
                store
                    .read(&account)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Enter the {role} scoped credential"))
            };
            let connected = ServerClient::new(
                &server,
                credential("material-submit", &submit)?,
                credential("worker", &worker)?,
            )?;
            // Reading a deliberately unknown inbox identity verifies connectivity and submit authority.
            let probe = connected
                .get::<serde_json::Value>(&format!("/api/v1/inbox/lessons/{}", uuid::Uuid::nil()));
            if let Err(problem) = probe
                && !problem.starts_with("Server 404")
            {
                return Err(problem);
            }
            *client = Some(connected);
            emit(Event::Connected(server));
            emit(Event::Status(
                "Connected; supplied lessons only, no model generation".into(),
            ));
            Ok(())
        }
        Command::Load(file) => {
            let bytes = pending::read_selected(&PathBuf::from(file), 8_000)?;
            let json =
                String::from_utf8(bytes).map_err(|_| "Prepared lesson must be UTF-8 JSON")?;
            crate::supplied::SuppliedLesson::parse(&json)?;
            emit(Event::Draft(json));
            emit(Event::Status(
                "Prepared text loaded locally; select its original audio and press Send".into(),
            ));
            Ok(())
        }
        command @ (Command::Send { .. } | Command::Retry) => {
            let client = client.as_ref().ok_or("Connect to the Server first")?;
            let paths = CoachPaths::for_current_user().map_err(|e| e.to_string())?;
            paths.ensure().map_err(|e| e.to_string())?;
            let path = paths.state.join("pending-send.json");
            let mut retained = match (pending::load(&path)?, command) {
                (Some(retained), Command::Retry) => retained,
                (Some(_), _) => {
                    return Err(
                        "A pending Send is retained. Retry it before sending another lesson".into(),
                    );
                }
                (None, Command::Send { json, audio }) => {
                    runner::create(&client.server, &json, &PathBuf::from(audio))?
                }
                _ => return Err("There is no pending Send to retry".into()),
            };
            let publication = runner::deliver(client, &path, &mut retained, |s| {
                emit(Event::Status(s.into()));
            })?;
            emit(Event::Published(format!(
                "{}/learn/en/phrases/{}",
                client.server,
                publication
                    .target_ids
                    .first()
                    .ok_or("Publication has no phrase")?
            )));
            emit(Event::Status(
                "Published; lesson and original audio are stored in Server".into(),
            ));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_reconnect_revokes_the_previous_clients_authority() {
        let mut client = Some(
            ServerClient::new("http://127.0.0.1:8000", "submit".into(), "worker".into()).unwrap(),
        );
        let problem = handle(
            Command::Connect {
                server: "https://example.com".into(),
                submit: String::new(),
                worker: String::new(),
            },
            &mut client,
            &|_| panic!("Failed connection must emit no success"),
        );
        assert!(problem.is_err());
        assert!(client.is_none());
        assert_eq!(
            handle(Command::Retry, &mut client, &|_| {}),
            Err("Connect to the Server first".into())
        );
    }
}
