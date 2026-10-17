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
    use std::path::{Path, PathBuf};

    use super::super::CliAdapter;
    use super::super::environment::HostEnvironment;
    use super::super::invocation::{AttemptFiles, Effort, ModelSelection};
    use super::*;

    fn build(model: &ModelSelection) -> Invocation {
        let root = std::env::temp_dir();
        let attempt = AttemptFiles::new(root.join("attempt Ünï Ђ"), "prompt.md").unwrap();
        let host = HostEnvironment::from_variables([
            (OsString::from("HOME"), OsString::from("/Users/learner")),
            (
                OsString::from("ANTHROPIC_API_KEY"),
                OsString::from("sk-ant"),
            ),
        ]);
        let executable = root.join("tools").join("claude");
        CliAdapter::Claude
            .invocation(&LaunchContext {
                executable: &executable,
                attempt: &attempt,
                model,
                host: &host,
                codex_home: Path::new("/unused"),
            })
            .unwrap()
    }

    #[test]
    fn runs_print_mode_with_a_file_bootstrap_after_the_option_separator() {
        let model = ModelSelection::new("haiku", None).unwrap();
        let invocation = build(&model);
        let args = invocation.arguments();
        let prompt = std::env::temp_dir().join("attempt Ünï Ђ").join("prompt.md");
        assert_eq!(&args[..3], ["-p", "--model", "haiku"]);
        assert_eq!(args[args.len() - 2], "--");
        assert_eq!(
            args[args.len() - 1],
            format!("Read your task from {} and execute it.", prompt.display())
        );
        assert_eq!(invocation.cwd, std::env::temp_dir().join("attempt Ünï Ђ"));
        assert_eq!(
            invocation.program,
            std::env::temp_dir().join("tools").join("claude")
        );
    }

    #[test]
    fn grants_only_read_and_loads_no_user_configuration() {
        let model = ModelSelection::new("haiku", Some(Effort::Low)).unwrap();
        let args = build(&model).arguments();
        let separator = args.iter().position(|argument| argument == "--").unwrap();
        let options = &args[..separator];
        assert_eq!(&options[3..5], ["--effort", "low"]);
        for required in ISOLATION {
            assert!(options.contains(&(*required).to_owned()), "{required}");
        }
        let tools = options.iter().position(|a| a == "--tools").unwrap();
        assert_eq!(options[tools + 1], "Read");
        let allowed = options.iter().position(|a| a == "--allowed-tools").unwrap();
        assert_eq!(options[allowed + 1], "Read");
        for forbidden in [
            "--bare",
            "--dangerously-skip-permissions",
            "--allow-dangerously-skip-permissions",
            "dontAsk",
            "bypassPermissions",
            "WebFetch",
            "WebSearch",
            "Bash",
            "Write",
            "--mcp-config",
            "--add-dir",
            "--chrome",
        ] {
            assert!(
                !options.iter().any(|argument| argument.contains(forbidden)),
                "{forbidden}"
            );
        }
    }

    #[test]
    fn strips_paid_api_credentials_and_disables_updates_and_connectors() {
        let model = ModelSelection::new("haiku", None).unwrap();
        let env = build(&model).env;
        let names: Vec<PathBuf> = env.iter().map(|(key, _)| PathBuf::from(key)).collect();
        assert!(!names.contains(&PathBuf::from("ANTHROPIC_API_KEY")));
        for (key, value) in [
            ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
            ("ENABLE_CLAUDEAI_MCP_SERVERS", "false"),
        ] {
            assert!(
                env.contains(&(OsString::from(key), OsString::from(value))),
                "{key}"
            );
        }
    }
}
