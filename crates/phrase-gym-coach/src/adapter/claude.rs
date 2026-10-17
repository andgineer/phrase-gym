use std::ffi::OsString;

use super::environment::child_environment;
use super::invocation::{
    Invocation, InvocationError, LaunchContext, arguments, bootstrap, checked_executable,
};

// --restricted drops user/project/local settings and CLAUDE.md, removes code-running tools and
// confines file tools to the working directory; --tools leaves Read as the only tool; --safe-mode
// disables plugins, hooks, skills and agents. Never --bare: it ignores the subscription login.
const ISOLATION: &[&str] = &[
    "--safe-mode",
    "--restricted",
    "--strict-mcp-config",
    "--tools",
    "Read",
    "--allowed-tools",
    "Read",
    "--permission-prompts",
    "none",
    "--no-session-persistence",
    "--disable-slash-commands",
];

// No background self-update during a pinned attempt, and no claude.ai connectors (remote MCP
// servers) fetched for the logged-in account.
const ENVIRONMENT: &[(&str, &str)] = &[
    ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
    ("ENABLE_CLAUDEAI_MCP_SERVERS", "false"),
];

pub(super) fn invocation(context: &LaunchContext<'_>) -> Result<Invocation, InvocationError> {
    let executable = checked_executable(context)?;
    let mut args = arguments(&["-p", "--model", context.model.model()]);
    if let Some(effort) = context.model.effort() {
        args.extend(arguments(&["--effort", effort.as_str()]));
    }
    args.extend(arguments(ISOLATION));
    // --allowed-tools is variadic; without the separator it would swallow the bootstrap.
    args.push(OsString::from("--"));
    args.push(OsString::from(bootstrap(context.attempt)?));
    let directory = executable.parent().unwrap_or(executable);
    let mut env = child_environment(context.host, directory);
    env.extend(
        ENVIRONMENT
            .iter()
            .map(|(key, value)| (OsString::from(key), OsString::from(value))),
    );
    Ok(Invocation {
        program: executable.to_path_buf(),
        args,
        cwd: context.attempt.directory().to_path_buf(),
        env,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::CliAdapter;
    use super::super::environment::HostEnvironment;
    use super::super::invocation::{AttemptFiles, Effort, ModelSelection};
    use super::*;

    #[cfg(unix)]
    const EXECUTABLE: &str = "/Users/learner/.local/share/claude/versions/2.1.302";
    #[cfg(unix)]
    const ATTEMPT: &str = "/Users/learner/Library/Caches/Coach Ünï/attempts/attempt Ђ";
    #[cfg(unix)]
    const BOOTSTRAP: &str = "Read your task from \
        /Users/learner/Library/Caches/Coach Ünï/attempts/attempt Ђ/prompt.md and execute it.";
    #[cfg(windows)]
    const EXECUTABLE: &str = r"C:\Users\learner\.local\bin\claude.exe";
    #[cfg(windows)]
    const ATTEMPT: &str = r"C:\Users\learner\AppData\Local\Coach Ünï\attempts\attempt Ђ";
    #[cfg(windows)]
    const BOOTSTRAP: &str = r"Read your task from C:\Users\learner\AppData\Local\Coach Ünï\attempts\attempt Ђ\prompt.md and execute it.";

    fn host() -> HostEnvironment {
        HostEnvironment::from_variables(
            [
                ("HOME", "/Users/learner"),
                ("USERPROFILE", r"C:\Users\learner"),
                ("ANTHROPIC_API_KEY", "sk-ant"),
                ("CLAUDECODE", "1"),
                ("PATH", "/Users/learner/.npm/bin:/usr/bin"),
            ]
            .map(|(key, value)| (OsString::from(key), OsString::from(value))),
        )
    }

    fn build(model: &ModelSelection) -> Invocation {
        let attempt = AttemptFiles::new(ATTEMPT.into(), "prompt.md").unwrap();
        CliAdapter::Claude
            .invocation(&LaunchContext {
                executable: Path::new(EXECUTABLE),
                attempt: &attempt,
                model,
                host: &host(),
                codex_home: Path::new("/unused"),
            })
            .unwrap()
    }

    #[test]
    fn passes_exactly_the_verified_arguments_without_effort() {
        let invocation = build(&ModelSelection::new("haiku", None).unwrap());
        assert_eq!(
            invocation.arguments(),
            [
                "-p",
                "--model",
                "haiku",
                "--safe-mode",
                "--restricted",
                "--strict-mcp-config",
                "--tools",
                "Read",
                "--allowed-tools",
                "Read",
                "--permission-prompts",
                "none",
                "--no-session-persistence",
                "--disable-slash-commands",
                "--",
                BOOTSTRAP,
            ]
        );
        assert_eq!(invocation.program, Path::new(EXECUTABLE));
        assert_eq!(invocation.cwd, Path::new(ATTEMPT));
    }

    #[test]
    fn passes_exactly_the_verified_arguments_with_effort() {
        let invocation = build(&ModelSelection::new("haiku", Some(Effort::Low)).unwrap());
        assert_eq!(
            invocation.arguments(),
            [
                "-p",
                "--model",
                "haiku",
                "--effort",
                "low",
                "--safe-mode",
                "--restricted",
                "--strict-mcp-config",
                "--tools",
                "Read",
                "--allowed-tools",
                "Read",
                "--permission-prompts",
                "none",
                "--no-session-persistence",
                "--disable-slash-commands",
                "--",
                BOOTSTRAP,
            ]
        );
    }

    #[test]
    fn adds_exactly_the_update_and_connector_switches_to_the_allowlisted_environment() {
        let env = build(&ModelSelection::new("haiku", None).unwrap()).env;
        let mut expected = child_environment(&host(), Path::new(EXECUTABLE).parent().unwrap());
        expected.extend(
            [
                ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
                ("ENABLE_CLAUDEAI_MCP_SERVERS", "false"),
            ]
            .map(|(key, value)| (OsString::from(key), OsString::from(value))),
        );
        assert_eq!(env, expected);
        assert!(
            !env.iter()
                .any(|(key, _)| key == "ANTHROPIC_API_KEY" || key == "CLAUDECODE")
        );
    }
}
