use super::state::{FolderHealth, ToolReadiness};

pub const CHECKING: &str = "Checking…";

pub fn intake_status(status: &str) -> Option<String> {
    if status.is_empty() {
        return None;
    }
    let message = if status.starts_with("Added ") {
        return Some(format!(
            "{} phrases added.",
            status.split_whitespace().nth(1).unwrap_or("Your")
        ));
    } else if status.starts_with("Accepted")
        || status.starts_with("Text accepted")
        || status.starts_with("Original text accepted")
        || status.starts_with("Preparing")
    {
        "Preparing your material…"
    } else if status.contains("without URLs") {
        "Add up to five short words or phrases. Paste text rather than a link."
    } else if status.contains("exceeded 90 seconds") {
        "Coach took too long to prepare this. Drop or paste it again to retry."
    } else if status.contains("quota") || status.contains("capacity limit") {
        "Preparation is temporarily unavailable. Wait for your provider to recover, then paste this material again."
    } else if status.contains("login required") {
        "Sign in to Codex, then paste this material again."
    } else if status.contains("failed validation")
        || [
            "Selected CLI frame",
            "Selected CLI accepted variants",
            "Selected CLI changed",
            "Selected CLI output changed",
            "Selected CLI returned",
        ]
        .iter()
        .any(|reason| status.contains(reason))
    {
        "Coach made an invalid card. Drop or paste the material again to retry."
    } else if status.contains("Server unavailable") || status.contains("connection") {
        "Coach can’t reach your library. Open Settings to check the connection."
    } else {
        "Coach couldn’t finish. Open Settings for details and retry."
    };
    Some(message.into())
}

pub fn processing(enabled: bool) -> &'static str {
    if enabled {
        "Enabled"
    } else {
        "Waiting for Server — processing resumes automatically"
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
    fn main_messages_show_progress_results_and_actions_without_diagnostics() {
        assert_eq!(intake_status(""), None);
        assert_eq!(
            intake_status("Text accepted durably by Server; preparation continues automatically")
                .as_deref(),
            Some("Preparing your material…")
        );
        assert_eq!(
            intake_status("Added 5 targets, confirmed by Server.").as_deref(),
            Some("5 phrases added.")
        );
        assert_eq!(
            intake_status(
                "Selected CLI preparation exceeded 90 seconds; accepted input remains on Server"
            )
            .as_deref(),
            Some("Coach took too long to prepare this. Drop or paste it again to retry.")
        );
        let message = intake_status("Prepared lesson failed validation: Server 422 Unprocessable Entity: Accepted variants must be distinct fixed wording").unwrap();
        assert!(message.contains("paste the material again"));
        assert!(!message.contains("Server"));
        assert!(!message.contains("422"));
    }

    #[test]
    fn tool_readiness_names_the_resolved_executable() {
        let found = ToolReadiness::Found(PathBuf::from("/opt/Tööls/codex"));
        assert_eq!(tool(&found), "Found at /opt/Tööls/codex");
        let script = ToolReadiness::Unsupported(PathBuf::from("/opt/node/codex.js"));
        assert!(tool(&script).starts_with("Unsupported install at /opt/node/codex.js"));
        assert!(!tool(&script).contains("Found"));
        assert_eq!(tool(&ToolReadiness::Checking), CHECKING);
        assert!(processing(false).starts_with("Waiting for Server"));
    }
}
