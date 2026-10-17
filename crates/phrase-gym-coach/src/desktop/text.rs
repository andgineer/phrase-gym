use super::state::{CredentialHealth, FolderHealth, ToolReadiness};

pub const CHECKING: &str = "Checking…";

pub fn processing(enabled: bool) -> &'static str {
    if enabled {
        "Enabled"
    } else {
        "Paused — nothing is claimed until processing is enabled"
    }
}

pub fn server(configured: bool) -> &'static str {
    if configured {
        "Configured"
    } else {
        "Not configured"
    }
}

pub fn tool(readiness: &ToolReadiness) -> String {
    match readiness {
        ToolReadiness::Checking => CHECKING.to_owned(),
        ToolReadiness::Found(path) => format!("Found at {}", path.display()),
        ToolReadiness::Unsupported(path) => format!(
            "Unsupported install at {}: a script that needs Node.js or another runtime. Install \
             the native build instead.",
            path.display()
        ),
        ToolReadiness::Missing => "Not found in its documented install locations".to_owned(),
    }
}

pub fn credentials(health: &CredentialHealth) -> String {
    match health {
        CredentialHealth::Checking => CHECKING.to_owned(),
        CredentialHealth::Ready {
            server_credential: true,
        } => "Available; a Server credential is stored".to_owned(),
        CredentialHealth::Ready {
            server_credential: false,
        } => "Available; no Server credential stored yet".to_owned(),
        CredentialHealth::Denied(reason) => format!(
            "Access denied ({reason}). Allow Phrase Gym Coach in the system credential store; \
             Coach never saves secrets anywhere else."
        ),
        CredentialHealth::Failed(reason) => {
            format!("Unavailable ({reason}). Coach never saves secrets anywhere else.")
        }
    }
}

pub fn folders(health: &FolderHealth) -> Vec<(&'static str, String)> {
    match health {
        FolderHealth::Checking => vec![("Folders", CHECKING.to_owned())],
        FolderHealth::Failed(reason) => vec![("Folders", format!("Cannot prepare: {reason}"))],
        FolderHealth::Ready(paths) => vec![
            ("Settings folder", paths.config.display().to_string()),
            ("Cache folder", paths.cache.display().to_string()),
            ("Log folder", paths.logs.display().to_string()),
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn denial_is_actionable_and_promises_no_other_secret_storage() {
        let text = credentials(&CredentialHealth::Denied("user canceled".to_owned()));
        assert!(text.starts_with("Access denied (user canceled)"));
        assert!(text.contains("never saves secrets anywhere else"));
        assert!(credentials(&CredentialHealth::Failed("no store".to_owned())).contains("no store"));
    }

    #[test]
    fn tool_readiness_names_the_resolved_executable() {
        let found = ToolReadiness::Found(PathBuf::from("/opt/Tööls/codex"));
        assert_eq!(tool(&found), "Found at /opt/Tööls/codex");
        let script = ToolReadiness::Unsupported(PathBuf::from("/opt/node/codex.js"));
        assert!(tool(&script).starts_with("Unsupported install at /opt/node/codex.js"));
        assert!(!tool(&script).contains("Found"));
        assert_eq!(tool(&ToolReadiness::Checking), CHECKING);
        assert!(processing(false).starts_with("Paused"));
    }
}
