use std::path::PathBuf;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use crate::adapter::CliAdapter;
use crate::platform::settings::Settings;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChoiceEvent {
    Loaded(CliAdapter),
    Saved(CliAdapter),
    Failed(String),
}

// Saving runs off the UI thread in request order, so a slow disk never stalls a frame and the
// last choice the learner made is the one that persists.
pub struct ToolChoiceWriter {
    requests: SyncSender<CliAdapter>,
    pub events: Receiver<ChoiceEvent>,
}

impl ToolChoiceWriter {
    pub fn spawn(path: PathBuf, wake: impl Fn() + Send + 'static) -> Self {
        let (requests, pending) = sync_channel::<CliAdapter>(4);
        let (send, events) = sync_channel(8);
        std::thread::spawn(move || {
            let report = |event| {
                if send.send(event).is_ok() {
                    wake();
                }
            };
            report(match Settings::load(&path) {
                Ok(saved) => ChoiceEvent::Loaded(saved.map_or(CliAdapter::Codex, |s| s.adapter)),
                Err(problem) => ChoiceEvent::Failed(problem),
            });
            while let Ok(adapter) = pending.recv() {
                report(match Settings::select_adapter(&path, adapter) {
                    Ok(()) => ChoiceEvent::Saved(adapter),
                    Err(problem) => ChoiceEvent::Failed(problem),
                });
            }
        });
        Self { requests, events }
    }

    pub fn select(&self, adapter: CliAdapter) -> Result<(), String> {
        self.requests
            .try_send(adapter)
            .map_err(|_| "Coach is still saving the previous choice; choose again".into())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn reports_the_saved_choice_then_persists_each_selection_in_order() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("settings.json");
        Settings::save(&path, "http://127.0.0.1:18487").unwrap();
        let writer = ToolChoiceWriter::spawn(path.clone(), || {});
        let next = || writer.events.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(next(), ChoiceEvent::Loaded(CliAdapter::Codex));
        writer.select(CliAdapter::Claude).unwrap();
        writer.select(CliAdapter::Codex).unwrap();
        writer.select(CliAdapter::Claude).unwrap();
        assert_eq!(next(), ChoiceEvent::Saved(CliAdapter::Claude));
        assert_eq!(next(), ChoiceEvent::Saved(CliAdapter::Codex));
        assert_eq!(next(), ChoiceEvent::Saved(CliAdapter::Claude));
        let saved = Settings::load(&path).unwrap().unwrap();
        assert_eq!(saved.adapter, CliAdapter::Claude);
        assert_eq!(saved.server, "http://127.0.0.1:18487");
    }

    #[test]
    fn unreadable_settings_are_reported_instead_of_silently_replaced() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("settings.json");
        std::fs::write(&path, b"{not json").unwrap();
        let writer = ToolChoiceWriter::spawn(path.clone(), || {});
        let event = writer.events.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(event, ChoiceEvent::Failed(problem) if problem.contains("invalid")));
        writer.select(CliAdapter::Claude).unwrap();
        let event = writer.events.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(event, ChoiceEvent::Failed(_)));
        assert_eq!(std::fs::read(&path).unwrap(), b"{not json");
    }
}
