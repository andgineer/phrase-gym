use std::path::{Path, PathBuf};

use crate::adapter::{
    CliAdapter, codex_readiness, environment::HostEnvironment, invocation::Effort,
};

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
            version: super::readiness::VERIFIED_VERSION,
            workspace: None,
            effort: None,
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

    pub fn attempts(&self, paths: &crate::platform::paths::CoachPaths) -> PathBuf {
        self.workspace.map_or_else(
            || paths.attempts(),
            |workspace| workspace.join(".phrase-gym-coach/attempts"),
        )
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
