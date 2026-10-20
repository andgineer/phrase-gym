use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use super::state::{FolderHealth, StartupEvent, ToolReadiness};
use crate::adapter::CliAdapter;
use crate::adapter::environment::HostEnvironment;
use crate::platform::paths::CoachPaths;
use crate::platform::tools;

type Check<T> = Box<dyn FnOnce() -> T + Send>;

// File-system and tool-discovery checks run off the UI thread and
// report through a channel; `wake` asks the UI to repaint once per result instead of polling.
pub struct StartupChecks {
    pub folders: Check<FolderHealth>,
    pub tool: Box<dyn Fn(CliAdapter) -> ToolReadiness + Send>,
}

impl StartupChecks {
    pub fn native() -> Self {
        let host = HostEnvironment::current();
        Self {
            folders: Box::new(folders),
            tool: Box::new(move |adapter| tools::discover(adapter, &host).into()),
        }
    }

    pub fn spawn(self, wake: impl Fn() + Send + 'static) -> Receiver<StartupEvent> {
        let (sender, receiver) = sync_channel(8);
        std::thread::spawn(move || self.run(&sender, &wake));
        receiver
    }

    fn run(self, sender: &SyncSender<StartupEvent>, wake: &dyn Fn()) {
        let send = |event| {
            if sender.send(event).is_ok() {
                wake();
            }
        };
        send(StartupEvent::Folders((self.folders)()));
        for adapter in CliAdapter::ALL {
            send(StartupEvent::Tool(adapter, (self.tool)(adapter)));
        }
    }
}

fn folders() -> FolderHealth {
    let prepared = CoachPaths::for_current_user().and_then(|paths| paths.ensure().map(|()| paths));
    match prepared {
        Ok(paths) => FolderHealth::Ready(paths),
        Err(error) => FolderHealth::Failed(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn reports_each_result_and_wakes_the_ui_once_per_result() {
        let checks = StartupChecks {
            folders: Box::new(|| FolderHealth::Failed("read-only disk".to_owned())),
            tool: Box::new(|adapter| match adapter {
                CliAdapter::Codex => ToolReadiness::Found(PathBuf::from("/opt/codex")),
                CliAdapter::Claude => ToolReadiness::Missing,
            }),
        };
        let wakes = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&wakes);
        let events: Vec<StartupEvent> = checks
            .spawn(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            })
            .iter()
            .collect();
        assert_eq!(
            events,
            [
                StartupEvent::Folders(FolderHealth::Failed("read-only disk".to_owned())),
                StartupEvent::Tool(
                    CliAdapter::Codex,
                    ToolReadiness::Found(PathBuf::from("/opt/codex"))
                ),
                StartupEvent::Tool(CliAdapter::Claude, ToolReadiness::Missing),
            ]
        );
        assert_eq!(wakes.load(Ordering::SeqCst), 3);
    }
}
