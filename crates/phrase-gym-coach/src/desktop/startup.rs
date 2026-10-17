use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use super::state::{CredentialHealth, FolderHealth, StartupEvent, ToolReadiness};
use crate::adapter::CliAdapter;
use crate::adapter::environment::HostEnvironment;
use crate::platform::credentials::{CredentialError, CredentialStore, SERVER_ACCOUNT};
use crate::platform::paths::CoachPaths;
use crate::platform::tools;

type Check<T> = Box<dyn FnOnce() -> T + Send>;

// Checks that may block on the file system or an OS credential prompt run off the UI thread and
// report through a channel; `wake` asks the UI to repaint once per result instead of polling.
pub struct StartupChecks {
    pub folders: Check<FolderHealth>,
    pub tool: Box<dyn Fn(CliAdapter) -> ToolReadiness + Send>,
    pub credentials: Check<CredentialHealth>,
}

impl StartupChecks {
    pub fn native() -> Self {
        let host = HostEnvironment::current();
        Self {
            folders: Box::new(folders),
            tool: Box::new(move |adapter| {
                tools::discover(adapter, &host).map_or(ToolReadiness::Missing, ToolReadiness::Found)
            }),
            credentials: Box::new(credentials),
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
        send(StartupEvent::Credentials((self.credentials)()));
    }
}

fn folders() -> FolderHealth {
    let prepared = CoachPaths::for_current_user().and_then(|paths| paths.ensure().map(|()| paths));
    match prepared {
        Ok(paths) => FolderHealth::Ready(paths),
        Err(error) => FolderHealth::Failed(error.to_string()),
    }
}

// Only reads: probing never creates an item, so a launch cannot silently persist a secret.
fn credentials() -> CredentialHealth {
    match CredentialStore::native().and_then(|store| store.read(SERVER_ACCOUNT)) {
        Ok(secret) => CredentialHealth::Ready {
            server_credential: secret.is_some(),
        },
        Err(CredentialError::Denied(reason)) => CredentialHealth::Denied(reason),
        Err(CredentialError::Failed(reason)) => CredentialHealth::Failed(reason),
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
            credentials: Box::new(|| CredentialHealth::Denied("keychain locked".to_owned())),
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
                StartupEvent::Credentials(CredentialHealth::Denied("keychain locked".to_owned())),
            ]
        );
        assert_eq!(wakes.load(Ordering::SeqCst), 4);
    }
}
