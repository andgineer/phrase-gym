use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{SyncSender, sync_channel};

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
    Summary(String),
    Connected(String),
    Configured(String),
    Published(String),
    Finished(Result<(), String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntakeState {
    pub server: String,
    pub status: String,
    pub summary: Option<String>,
    pub connected: bool,
    pub busy: bool,
    pub practice: Option<String>,
    pub command: Option<Command>,
}

impl Default for IntakeState {
    fn default() -> Self {
        Self {
            server: phrase_gym_contracts::connection::default_server_origin(),
            summary: None,
            status: String::new(),
            connected: false,
            busy: false,
            practice: None,
            command: None,
        }
    }
}

pub struct IntakeRunner {
    pub commands: SyncSender<Command>,
    pub events: super::intake_events::Events,
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
        let events = super::intake_events::Events::default();
        let send = events.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&shutdown);
        let thread = std::thread::spawn(move || {
            let emit = |event| {
                send.push(event);
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
