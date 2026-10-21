use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Connect { server: String },
    Text(String),
    DropText(std::path::PathBuf),
    Retry,
    RetryPreparation,
}

pub enum Event {
    Status(String),
    Draft(String),
    Connected(String),
    SelectedClaude(std::path::PathBuf),
    Published(String),
    Finished(Result<(), String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntakeState {
    pub server: String,
    pub status: String,
    pub connected: bool,
    pub busy: bool,
    pub practice: Option<String>,
    pub command: Option<Command>,
}

impl Default for IntakeState {
    fn default() -> Self {
        Self {
            server: phrase_gym_contracts::connection::default_server_origin(),
            status: "Connecting automatically using saved Server access…".into(),
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
    shutdown: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl IntakeRunner {
    pub fn spawn(
        wake: impl Fn() + Send + 'static,
        processing: bool,
        smoke: Option<super::smoke_intake::SmokeInput>,
    ) -> Self {
        let (commands, receive) = sync_channel(2);
        let (send, events) = sync_channel(32);
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&shutdown);
        let thread = std::thread::spawn(move || {
            let emit = |event| {
                // A closed/minimized UI must never block lease cleanup or shutdown.
                let _ = send.try_send(event);
                wake();
            };
            if let Some(input) = smoke {
                emit(Event::Finished(super::smoke_intake::run(
                    &input, &emit, &stop,
                )));
            } else {
                super::intake_runtime::run(&receive, &emit, &stop, processing);
            }
        });
        Self {
            commands,
            events,
            shutdown,
            thread: Some(thread),
        }
    }
}

impl Drop for IntakeRunner {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
