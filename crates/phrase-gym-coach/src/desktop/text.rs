use super::state::{FolderHealth, ToolReadiness};
use crate::adapter::CliAdapter;
use crate::transfer::input_problem::InputProblem;

pub const CHECKING: &str = "Checking…";

pub fn intake_status(status: &str, tool: Option<CliAdapter>) -> Option<String> {
    if status.is_empty() {
        return None;
    }
    if let Some(problem) = InputProblem::recognize(status) {
        return Some(problem.message().to_owned());
    }
    let tool = tool.map_or("your preparation tool", CliAdapter::short_name);
    let message = if status.starts_with("Added:") {
        return Some(status.to_owned());
    } else if status.starts_with("Accepted")
        || status.starts_with("Text accepted")
        || status.starts_with("Original text accepted")
        || status.starts_with("Preparing")
    {
        "Preparing your material…".to_owned()
    } else if status.contains("exceeded 90 seconds") {
        format!("{tool} took too long to prepare this. Drop or paste it again to retry.")
    } else if status.contains("quota") || status.contains("capacity limit") {
        format!(
            "{tool} is temporarily unavailable or out of quota. Wait for it to recover, then paste this material again."
        )
    } else if status.contains("login required") {
        format!("Sign in to {tool} outside Coach, then paste this material again.")
    } else if status.contains("failed validation")
        || status.contains("stopped without a valid answer")
        || status.contains("without the required structured answer")
        || [
            "Selected CLI frame",
            "Selected CLI accepted variants",
            "Selected CLI changed",
            "Selected CLI output changed",
            "Selected CLI returned",
            "Selected CLI omitted",
            "Claude returned",
        ]
        .iter()
        .any(|reason| status.contains(reason))
    {
        format!("{tool} made an invalid card. Drop or paste the material again to retry.")
    } else if status.contains("Server unavailable")
        || status.contains("connection")
        || status.contains("Cannot reach")
    {
        "Coach can’t reach your library. Open Settings to check the connection.".to_owned()
    } else {
        let reason: String = status.chars().take(160).collect();
        let reason = reason.trim_end_matches(['.', ' ']);
        format!("Coach couldn’t finish: {reason}. Open Settings for details and retry.")
    };
    Some(message)
}

// Names what was published: a word by its word, other items by their text.
pub fn confirmed_additions(
    output: Option<&phrase_gym_contracts::inbox::LessonOutput>,
    published: usize,
) -> String {
    let names: Vec<&str> = output.map_or_else(Vec::new, |output| {
        output
            .package
            .phrases
            .iter()
            .map(|phrase| match &phrase.task {
                Some(phrase_gym_contracts::english::EnglishTask::WordSense { word, .. }) => {
                    word.as_str()
                }
                _ => phrase.text.as_str(),
            })
            .collect()
    });
    if names.is_empty() {
        format!("Added: {published} items")
    } else {
        format!("Added: {}", crate::transfer::items::lesson_title(&names))
    }
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
        let codex = Some(CliAdapter::Codex);
        assert_eq!(intake_status("", codex), None);
        assert_eq!(
            intake_status(
                "Text accepted durably by Server; preparation continues automatically",
                codex
            )
            .as_deref(),
            Some("Preparing your material…")
        );
        assert_eq!(
            intake_status("Added: impediment", codex).as_deref(),
            Some("Added: impediment")
        );
        assert_eq!(
            intake_status(
                "Selected CLI preparation exceeded 90 seconds; accepted input remains on Server",
                Some(CliAdapter::Claude)
            )
            .as_deref(),
            Some("Claude took too long to prepare this. Drop or paste it again to retry.")
        );
        let message = intake_status("Prepared lesson failed validation: Server 422 Unprocessable Entity: Accepted variants must be distinct fixed wording", codex).unwrap();
        assert!(message.contains("paste the material again"));
        assert!(!message.contains("Server") && !message.contains("422"));
        assert_eq!(
            intake_status("Selected CLI subscription login required; sign in outside Coach, then retry preparation", Some(CliAdapter::Claude)).as_deref(),
            Some("Sign in to Claude outside Coach, then paste this material again.")
        );
    }

    #[test]
    fn every_learner_input_problem_reaches_the_main_window_verbatim() {
        for problem in InputProblem::ALL {
            assert_eq!(
                intake_status(problem.message(), None).as_deref(),
                Some(problem.message())
            );
        }
    }

    #[test]
    fn confirmed_additions_name_words_by_their_word_and_phrases_by_their_text() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/coach/supplied-lesson.json"
        ))
        .unwrap();
        let mut phrase: phrase_gym_contracts::english::PreparedEnglishPhrase =
            serde_json::from_value(fixture["phrase"].clone()).unwrap();
        let mut output: phrase_gym_contracts::inbox::LessonOutput =
            serde_json::from_value(serde_json::json!({
                "fence": {"generation_id": uuid::Uuid::nil(), "claim_id": uuid::Uuid::nil(), "token": uuid::Uuid::nil(), "fence": 1},
                "publication_id": uuid::Uuid::nil(), "adapter": "synthetic",
                "package": {"schema_version": "english_phrases_v1", "namespace": "n", "source_revision": "1", "language": "en", "profile_revision": 1, "guide_version": "g", "assessment_version": "a",
                    "lesson": {"source_id": "l", "title": "t", "target_source_ids": []}, "phrases": []}
            }))
            .unwrap();
        output.package.phrases.push(phrase.clone());
        phrase.text = "A lack of funding became a major impediment to the project.".into();
        phrase.task = Some(phrase_gym_contracts::english::EnglishTask::WordSense {
            word: "impediment".into(),
            sense: "something that makes progress difficult".into(),
            contexts: Vec::new(),
        });
        output.package.phrases.push(phrase);
        assert_eq!(
            confirmed_additions(Some(&output), 2),
            "Added: Let's get started. · impediment"
        );
        assert_eq!(confirmed_additions(None, 3), "Added: 3 items");
    }

    #[test]
    fn only_unexpected_failures_use_the_catch_all_and_it_names_the_reason() {
        let message = intake_status(
            "Cannot create protected protected local state; preserve any existing temporary file",
            None,
        )
        .unwrap();
        assert_eq!(
            message,
            "Coach couldn’t finish: Cannot create protected protected local state; preserve any existing temporary file. Open Settings for details and retry."
        );
        let long = intake_status(&"x".repeat(500), None).unwrap();
        assert!(long.chars().count() < 260);
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
