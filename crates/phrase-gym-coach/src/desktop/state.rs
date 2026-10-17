use std::path::PathBuf;

use crate::adapter::CliAdapter;
use crate::platform::paths::CoachPaths;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolReadiness {
    Checking,
    Found(PathBuf),
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialHealth {
    Checking,
    Ready { server_credential: bool },
    Denied(String),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FolderHealth {
    Checking,
    Ready(CoachPaths),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartupEvent {
    Folders(FolderHealth),
    Tool(CliAdapter, ToolReadiness),
    Credentials(CredentialHealth),
}

// Every launch starts paused and unconnected; nothing is claimed until the user enables processing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub processing_enabled: bool,
    pub server_configured: bool,
    pub tools: Vec<(CliAdapter, ToolReadiness)>,
    pub credentials: CredentialHealth,
    pub folders: FolderHealth,
}

impl StatusSnapshot {
    pub fn starting() -> Self {
        Self {
            processing_enabled: false,
            server_configured: false,
            tools: CliAdapter::ALL
                .iter()
                .map(|adapter| (*adapter, ToolReadiness::Checking))
                .collect(),
            credentials: CredentialHealth::Checking,
            folders: FolderHealth::Checking,
        }
    }

    pub fn apply(&mut self, event: StartupEvent) {
        match event {
            StartupEvent::Folders(health) => self.folders = health,
            StartupEvent::Credentials(health) => self.credentials = health,
            StartupEvent::Tool(adapter, readiness) => {
                for (known, state) in &mut self.tools {
                    if *known == adapter {
                        *state = readiness.clone();
                    }
                }
            }
        }
    }

    pub fn settled(&self) -> bool {
        self.credentials != CredentialHealth::Checking
            && self.folders != FolderHealth::Checking
            && self
                .tools
                .iter()
                .all(|(_, readiness)| *readiness != ToolReadiness::Checking)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewState {
    pub snapshot: StatusSnapshot,
    pub theme: ThemeChoice,
    pub text_scale: f32,
    pub draft: String,
}

impl ViewState {
    pub fn new(snapshot: StatusSnapshot) -> Self {
        Self {
            snapshot,
            theme: ThemeChoice::System,
            text_scale: 1.0,
            draft: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launches_paused_and_settles_once_every_check_reports() {
        let mut snapshot = StatusSnapshot::starting();
        assert!(!snapshot.processing_enabled && !snapshot.server_configured);
        assert!(!snapshot.settled());
        snapshot.apply(StartupEvent::Tool(
            CliAdapter::Codex,
            ToolReadiness::Found(PathBuf::from("/opt/codex")),
        ));
        snapshot.apply(StartupEvent::Tool(
            CliAdapter::Claude,
            ToolReadiness::Missing,
        ));
        snapshot.apply(StartupEvent::Credentials(CredentialHealth::Denied(
            "locked".to_owned(),
        )));
        assert!(!snapshot.settled());
        snapshot.apply(StartupEvent::Folders(FolderHealth::Failed(
            "disk".to_owned(),
        )));
        assert!(snapshot.settled());
        assert_eq!(
            snapshot.tools,
            [
                (
                    CliAdapter::Codex,
                    ToolReadiness::Found(PathBuf::from("/opt/codex"))
                ),
                (CliAdapter::Claude, ToolReadiness::Missing),
            ]
        );
        assert!(!snapshot.processing_enabled);
    }
}
