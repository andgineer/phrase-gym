use serde::{Deserialize, Serialize};

// Persisted as a closed choice so settings can never smuggle in an arbitrary shell command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CliAdapter {
    Codex,
    Claude,
}

impl CliAdapter {
    pub fn executable_name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
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
