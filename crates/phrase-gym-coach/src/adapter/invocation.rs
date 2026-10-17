use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use super::environment::HostEnvironment;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InvocationError {
    #[error("{0} must be an absolute path")]
    RelativePath(&'static str),
    #[error("{0} must be valid Unicode to be passed to the CLI")]
    NonUnicodePath(&'static str),
    #[error("the prompt file name must be a plain file name inside the attempt directory")]
    PromptOutsideAttempt,
    #[error("model name {0:?} is not a plain identifier")]
    InvalidModel(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effort {
    Low,
    Medium,
    High,
}

impl Effort {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelSelection {
    model: String,
    effort: Option<Effort>,
}

impl ModelSelection {
    pub fn new(model: &str, effort: Option<Effort>) -> Result<Self, InvocationError> {
        let plain = model
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character));
        if model.is_empty() || model.starts_with('-') || !plain {
            return Err(InvocationError::InvalidModel(model.to_owned()));
        }
        Ok(Self {
            model: model.to_owned(),
            effort,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn effort(&self) -> Option<Effort> {
        self.effort
    }
}

// The CLI receives only a bootstrap naming this file, so the prompt never travels through stdin
// or an argument where length and quoting limits would apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttemptFiles {
    directory: PathBuf,
    prompt: PathBuf,
}

impl AttemptFiles {
    pub fn new(directory: PathBuf, prompt_name: &str) -> Result<Self, InvocationError> {
        if !directory.is_absolute() {
            return Err(InvocationError::RelativePath("attempt directory"));
        }
        let mut components = Path::new(prompt_name).components();
        let plain =
            matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
        if !plain {
            return Err(InvocationError::PromptOutsideAttempt);
        }
        let prompt = directory.join(prompt_name);
        Ok(Self { directory, prompt })
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn prompt(&self) -> &Path {
        &self.prompt
    }
}

pub struct LaunchContext<'a> {
    // The resolved, canonical executable: Codex re-executes itself by this path inside its sandbox.
    pub executable: &'a Path,
    pub attempt: &'a AttemptFiles,
    pub model: &'a ModelSelection,
    pub host: &'a HostEnvironment,
    // Coach-owned Codex home holding only Coach's Codex login; ignored by other adapters.
    pub codex_home: &'a Path,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    pub env: Vec<(OsString, OsString)>,
}

impl Invocation {
    pub fn arguments(&self) -> Vec<String> {
        self.args
            .iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }
}

pub(super) fn checked_executable<'a>(
    context: &LaunchContext<'a>,
) -> Result<&'a Path, InvocationError> {
    let executable = context.executable;
    if !executable.is_absolute() {
        return Err(InvocationError::RelativePath("CLI executable"));
    }
    Ok(executable)
}

pub(super) fn bootstrap(attempt: &AttemptFiles) -> Result<String, InvocationError> {
    let prompt = attempt
        .prompt()
        .to_str()
        .ok_or(InvocationError::NonUnicodePath("prompt path"))?;
    Ok(format!("Read your task from {prompt} and execute it."))
}

pub(super) fn arguments(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_must_be_a_plain_file_inside_an_absolute_attempt_directory() {
        let root = std::env::temp_dir().join("attempt Ünï Ђ");
        let attempt = AttemptFiles::new(root.clone(), "prompt.md").unwrap();
        assert_eq!(attempt.prompt(), root.join("prompt.md"));
        for escaping in ["../prompt.md", "nested/prompt.md", "", "/etc/passwd", "."] {
            assert_eq!(
                AttemptFiles::new(root.clone(), escaping),
                Err(InvocationError::PromptOutsideAttempt),
                "{escaping:?}"
            );
        }
        assert_eq!(
            AttemptFiles::new(PathBuf::from("relative"), "prompt.md"),
            Err(InvocationError::RelativePath("attempt directory"))
        );
    }

    #[test]
    fn model_names_cannot_inject_flags_or_shell_syntax() {
        assert!(ModelSelection::new("gpt-5.6-luna", Some(Effort::Low)).is_ok());
        assert!(ModelSelection::new("claude-haiku-4-5_20251001", None).is_ok());
        for invalid in [
            "",
            "--dangerously-skip-permissions",
            "haiku; rm -rf",
            "a b",
            "m\"x",
        ] {
            assert!(ModelSelection::new(invalid, None).is_err(), "{invalid:?}");
        }
    }
}
