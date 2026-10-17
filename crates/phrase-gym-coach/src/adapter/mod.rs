mod claude;
mod codex;
pub mod environment;
pub mod invocation;

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
