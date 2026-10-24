use std::path::{Path, PathBuf};

use crate::adapter::{
    CliAdapter, codex_readiness, environment::HostEnvironment, invocation::Effort,
};
use crate::platform::paths::CoachPaths;

const WORKSPACE_ATTEMPTS: &str = ".phrase-gym-coach/attempts";

pub struct SelectedTool<'a> {
    pub adapter: CliAdapter,
    pub executable: &'a Path,
    pub host: &'a HostEnvironment,
    pub home: Option<&'a Path>,
    pub model: &'a str,
    pub version: &'a str,
    pub workspace: Option<&'a Path>,
    pub effort: Option<Effort>,
}

impl<'a> SelectedTool<'a> {
    pub fn claude(tool: &'a super::readiness::ClaudeTool) -> Self {
        Self {
            adapter: CliAdapter::Claude,
            executable: &tool.executable,
            host: &tool.host,
            home: None,
            model: super::readiness::MODEL,
            version: &tool.version,
            workspace: None,
            effort: Some(Effort::Low),
        }
    }

    pub fn codex(tool: &'a codex_readiness::CodexReady, workspace: &'a Path) -> Self {
        Self {
            adapter: CliAdapter::Codex,
            executable: &tool.executable,
            host: &tool.host,
            home: Some(&tool.home),
            model: codex_readiness::MODEL,
            version: codex_readiness::VERIFIED_VERSION,
            workspace: Some(workspace),
            effort: Some(Effort::Low),
        }
    }

    pub fn attempts(&self, paths: &CoachPaths) -> PathBuf {
        self.workspace.map_or_else(
            || paths.attempts(),
            |workspace| workspace.join(WORKSPACE_ATTEMPTS),
        )
    }

    // A fenced attempt interrupted under one tool must not start again after switching tools.
    pub fn started_attempt(&self, paths: &CoachPaths, token: &str) -> Option<PathBuf> {
        let mut folders = vec![self.attempts(paths), paths.attempts()];
        if let Ok(workspace) = super::workspace::load(paths) {
            folders.push(workspace.join(WORKSPACE_ATTEMPTS));
        }
        folders
            .into_iter()
            .map(|folder| folder.join(token))
            .find(|attempt| attempt.exists())
    }

    pub fn identity(&self) -> String {
        format!(
            "{}-{}-{}-demo-v1",
            self.adapter.executable_name(),
            self.version,
            self.model
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_attempt_started_by_either_tool_is_found_after_switching() {
        let root = tempfile::tempdir().unwrap();
        let paths = CoachPaths {
            config: root.path().join("config"),
            cache: root.path().join("cache"),
            logs: root.path().join("logs"),
            state: root.path().join("state"),
        };
        paths.ensure().unwrap();
        let workspace = root.path().join("repository");
        std::fs::create_dir_all(workspace.join(".git")).unwrap();
        std::fs::write(workspace.join("Cargo.toml"), "").unwrap();
        let workspace = super::super::workspace::save(&paths, &workspace).unwrap();
        let host = HostEnvironment::default();
        let claude = super::super::readiness::ClaudeTool {
            executable: PathBuf::from("/opt/claude"),
            host: host.clone(),
            version: "2.1.309".into(),
        };
        let codex = codex_readiness::CodexReady {
            executable: PathBuf::from("/opt/codex"),
            home: root.path().join("home"),
            host,
        };
        let claude = SelectedTool::claude(&claude);
        let codex = SelectedTool::codex(&codex, &workspace);
        assert_eq!(claude.started_attempt(&paths, "fence-1"), None);
        let by_codex = workspace.join(WORKSPACE_ATTEMPTS).join("fence-1");
        std::fs::create_dir_all(&by_codex).unwrap();
        assert_eq!(claude.started_attempt(&paths, "fence-1"), Some(by_codex));
        let by_claude = paths.attempts().join("fence-2");
        std::fs::create_dir_all(&by_claude).unwrap();
        assert_eq!(codex.started_attempt(&paths, "fence-2"), Some(by_claude));
        assert_eq!(codex.started_attempt(&paths, "fence-3"), None);
    }
}
