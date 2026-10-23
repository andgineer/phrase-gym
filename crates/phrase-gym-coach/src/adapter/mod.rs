pub mod claude;
pub mod claude_output;
mod codex;
pub mod codex_readiness;
pub mod environment;
pub mod invocation;

use std::ffi::OsString;
use std::path::Path;

use serde::{Deserialize, Serialize};

use invocation::{Invocation, InvocationError, LaunchContext};

// Persisted as a closed choice so settings can never smuggle in an arbitrary shell command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CliAdapter {
    Codex,
    Claude,
}

impl CliAdapter {
    pub const ALL: [Self; 2] = [Self::Codex, Self::Claude];

    pub fn executable_name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::Claude => "Claude",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Codex => "Codex CLI",
            Self::Claude => "Claude CLI",
        }
    }

    pub fn invocation(self, context: &LaunchContext<'_>) -> Result<Invocation, InvocationError> {
        match self {
            Self::Codex => codex::invocation(context),
            Self::Claude => claude::invocation(context),
        }
    }

    // Each CLI enforces the staged output schema with its own flags.
    pub fn require_schema(self, invocation: &mut Invocation, path: &Path, schema: &str) {
        match self {
            Self::Codex => {
                let at = invocation.args.len().saturating_sub(1);
                invocation.args.splice(
                    at..at,
                    [OsString::from("--output-schema"), path.as_os_str().into()],
                );
            }
            Self::Claude => claude::require_schema(invocation, schema),
        }
    }

    pub fn answer(self, stdout: &[u8]) -> Result<serde_json::Value, String> {
        match self {
            Self::Codex => serde_json::from_slice(stdout).map_err(|_| {
                "Selected CLI returned empty or malformed output; inspect the private attempt"
                    .into()
            }),
            Self::Claude => claude_output::answer(stdout),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_accept_only_the_two_supported_adapters() {
        let selected: CliAdapter = serde_json::from_str("\"claude\"").unwrap();
        assert_eq!(selected.executable_name(), "claude");
        assert_eq!(
            serde_json::to_string(&CliAdapter::Codex).unwrap(),
            "\"codex\""
        );
        assert!(serde_json::from_str::<CliAdapter>("\"codex exec --dangerous\"").is_err());
    }
}
