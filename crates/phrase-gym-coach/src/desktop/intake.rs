use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use super::preview::LessonPreview;
use crate::transfer::material::RawLesson;

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
    Raw {
        title: String,
        phrase: String,
        situation: String,
        performer: String,
        locale: String,
        audio: String,
    },
    Retry,
    RetryPreparation,
    Load(String),
    ChooseLesson,
    ChooseAudio,
    PreviewAudio(String),
    Demo,
}

pub enum Event {
    Status(String),
    Connected(String),
    SelectedClaude(std::path::PathBuf),
    Draft(String),
    Audio(String),
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
    pub prepared: Option<String>,
    pub preview: Option<LessonPreview>,
    pub title: String,
    pub situation: String,
    pub performer: String,
    pub locale: String,
}

impl Default for IntakeState {
    fn default() -> Self {
        Self {
            server: "http://127.0.0.1:8000".into(),
            submit: String::new(),
            worker: String::new(),
            audio: String::new(),
            lesson_file: String::new(),
            status: "Connecting automatically using saved Server access…".into(),
            connected: false,
            busy: false,
            practice: None,
            command: None,
            prepared: None,
            preview: None,
            title: String::new(),
            situation: String::new(),
            performer: String::new(),
            locale: "en-US".into(),
        }
    }
}

impl IntakeState {
    pub fn accept_audio(&mut self, path: String, phrase: &str) {
        self.audio = path;
        self.command = Some(if let Some(json) = &self.prepared {
            Command::Send {
                json: json.clone(),
                audio: self.audio.clone(),
            }
        } else {
            Command::Raw {
                title: self.title.clone(),
                phrase: phrase.into(),
                situation: self.situation.clone(),
                performer: self.performer.clone(),
                locale: self.locale.clone(),
                audio: self.audio.clone(),
            }
        });
    }

    pub fn raw(command: &Command) -> Option<RawLesson> {
        let Command::Raw {
            title,
            phrase,
            situation,
            performer,
            locale,
            ..
        } = command
        else {
            return None;
        };
        Some(RawLesson {
            title: title.clone(),
            phrase: phrase.clone(),
            situation: situation.clone(),
            performer: performer.clone(),
            locale: locale.clone(),
        })
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
