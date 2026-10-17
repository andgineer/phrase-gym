use std::ffi::OsString;
use std::fmt::Write as _;
use std::path::Path;

use super::environment::child_environment;
use super::invocation::{
    Invocation, InvocationError, LaunchContext, arguments, bootstrap, checked_executable,
};

const PROFILE: &str = "coach_job";
// --ignore-user-config skips config.toml (MCP servers, plugins, notify, model) but not the home's
// AGENTS.md or skills, which is why Coach points CODEX_HOME at a home it owns.
const ISOLATION: &[&str] = &[
    "exec",
    "--skip-git-repo-check",
    "--ignore-user-config",
    "--ignore-rules",
    "--ephemeral",
];
// view_image reads files from the Codex process itself rather than a sandboxed command, and goals
// persist state in the Codex home; neither is needed by a preparation attempt.
const DISABLED_FEATURES: &[&str] = &[
    "apps",
    "browser_use",
    "browser_use_external",
    "computer_use",
    "goals",
    "hooks",
    "image_generation",
    "in_app_browser",
    "multi_agent",
    "plugins",
    "remote_plugin",
    "skill_mcp_dependency_install",
    "tool_suggest",
    "view_image",
];
// forced_login_method refuses an API-key login in the Codex home, so a stored key can never turn
// a subscription attempt into paid API usage.
const SETTINGS: &[&str] = &[
    "forced_login_method=\"chatgpt\"",
    "approval_policy=\"never\"",
    "web_search=\"disabled\"",
    "history.persistence=\"none\"",
    "check_for_update_on_startup=false",
];

pub(super) fn invocation(context: &LaunchContext<'_>) -> Result<Invocation, InvocationError> {
    let executable = checked_executable(context)?;
    let directory = executable.parent().unwrap_or(executable);
    if !context.codex_home.is_absolute() {
        return Err(InvocationError::RelativePath("Codex home"));
    }
    let mut args = arguments(ISOLATION);
    // A permission profile instead of --sandbox: the legacy workspace-write sandbox lets commands
    // read the whole disk and write /tmp, and passing --sandbox disables profiles entirely.
    args.extend(arguments(&[
        "-c",
        &format!("default_permissions=\"{PROFILE}\""),
    ]));
    args.extend(arguments(&["-c", &permission_profile(directory)?]));
    for setting in SETTINGS {
        args.extend(arguments(&["-c", setting]));
    }
    for feature in DISABLED_FEATURES {
        args.extend(arguments(&["--disable", feature]));
    }
    args.extend(arguments(&["--model", context.model.model()]));
    if let Some(effort) = context.model.effort() {
        let setting = format!("model_reasoning_effort={}", effort.as_str());
        args.extend(arguments(&["-c", &setting]));
    }
    args.push(OsString::from(bootstrap(context.attempt)?));
    let mut env = child_environment(context.host, directory);
    env.push((
        OsString::from("CODEX_HOME"),
        context.codex_home.as_os_str().to_os_string(),
    ));
    Ok(Invocation {
        program: executable.to_path_buf(),
        args,
        cwd: context.attempt.directory().to_path_buf(),
        env,
    })
}

// Commands may read only platform runtime paths and Codex's own install directory, and may write
// only the attempt directory; /tmp and $TMPDIR are closed explicitly because :minimal opens them.
fn permission_profile(executable_directory: &Path) -> Result<String, InvocationError> {
    let install = executable_directory
        .to_str()
        .ok_or(InvocationError::NonUnicodePath("CLI executable"))?;
    Ok(format!(
        "permissions.{PROFILE}={{filesystem={{\":root\"=\"deny\", \":minimal\"=\"read\", \
         \":slash_tmp\"=\"deny\", \":tmpdir\"=\"deny\", {}=\"read\", \
         \":workspace_roots\"={{\".\"=\"write\"}}}}, network={{enabled=false}}}}",
        toml_string(install)
    ))
}

fn toml_string(value: &str) -> String {
    let mut quoted = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            control if control.is_control() => {
                let _ = write!(quoted, "\\u{:04X}", u32::from(control));
            }
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::CliAdapter;
    use super::super::environment::HostEnvironment;
    use super::super::invocation::{AttemptFiles, Effort, ModelSelection};
    use super::*;

    fn build(executable: &Path) -> Invocation {
        let root = std::env::temp_dir();
        let attempt = AttemptFiles::new(root.join("attempt Ünï Ђ"), "prompt.md").unwrap();
        let model = ModelSelection::new("gpt-5.6-luna", Some(Effort::Low)).unwrap();
        let host = HostEnvironment::from_variables([
            (
                OsString::from("OPENAI_API_KEY"),
                OsString::from("sk-openai"),
            ),
            (
                OsString::from("CODEX_HOME"),
                OsString::from("/Users/learner/.codex"),
            ),
        ]);
        CliAdapter::Codex
            .invocation(&LaunchContext {
                executable,
                attempt: &attempt,
                model: &model,
                host: &host,
                codex_home: &root.join("Coach codex home"),
            })
            .unwrap()
    }

    fn option_values<'a>(args: &'a [String], flag: &str) -> Vec<&'a str> {
        args.windows(2)
            .filter(|pair| pair[0] == flag)
            .map(|pair| pair[1].as_str())
            .collect()
    }

    #[test]
    fn runs_exec_outside_git_with_a_file_bootstrap_and_low_effort() {
        let executable = std::env::temp_dir().join("Caskroom").join("codex");
        let args = build(&executable).arguments();
        assert_eq!(&args[..5], ISOLATION);
        assert!(args.contains(&"--skip-git-repo-check".to_owned()));
        let prompt = std::env::temp_dir().join("attempt Ünï Ђ").join("prompt.md");
        assert_eq!(
            args.last().unwrap(),
            &format!("Read your task from {} and execute it.", prompt.display())
        );
        assert_eq!(option_values(&args, "--model"), ["gpt-5.6-luna"]);
        assert!(option_values(&args, "-c").contains(&"model_reasoning_effort=low"));
    }

    #[test]
    fn confines_commands_with_a_permission_profile_and_disables_extensions() {
        let executable = std::env::temp_dir().join("Caskroom").join("codex");
        let args = build(&executable).arguments();
        let settings = option_values(&args, "-c");
        assert!(settings.contains(&"default_permissions=\"coach_job\""));
        let profile = settings
            .iter()
            .find(|setting| setting.starts_with("permissions.coach_job="))
            .unwrap();
        for rule in [
            "\":root\"=\"deny\"",
            "\":slash_tmp\"=\"deny\"",
            "\":tmpdir\"=\"deny\"",
            "\":workspace_roots\"={\".\"=\"write\"}",
            "network={enabled=false}",
        ] {
            assert!(profile.contains(rule), "{rule} missing from {profile}");
        }
        assert!(settings.contains(&"web_search=\"disabled\""));
        assert!(settings.contains(&"forced_login_method=\"chatgpt\""));
        assert_eq!(option_values(&args, "--disable"), DISABLED_FEATURES);
        for forbidden in [
            "--sandbox",
            "--dangerously-bypass-approvals-and-sandbox",
            "--full-auto",
            "danger-full-access",
            "network_access=true",
            "--profile",
        ] {
            assert!(!args.iter().any(|a| a.contains(forbidden)), "{forbidden}");
        }
    }

    #[test]
    fn uses_the_coach_codex_home_and_strips_api_keys() {
        let executable = std::env::temp_dir().join("Caskroom").join("codex");
        let invocation = build(&executable);
        let home = invocation
            .env
            .iter()
            .filter(|(key, _)| key == "CODEX_HOME")
            .map(|(_, value)| PathBuf::from(value))
            .collect::<Vec<_>>();
        assert_eq!(home, [std::env::temp_dir().join("Coach codex home")]);
        assert!(
            !invocation
                .env
                .iter()
                .any(|(key, _)| key == "OPENAI_API_KEY")
        );
    }

    #[test]
    fn quotes_install_paths_as_toml_strings() {
        assert_eq!(
            toml_string(r"C:\Program Files\Codex"),
            r#""C:\\Program Files\\Codex""#
        );
        assert_eq!(toml_string("a\"b\nc"), r#""a\"b\u000Ac""#);
        let executable = std::env::temp_dir().join("Ünï \"tools\"").join("codex");
        let args = build(&executable).arguments();
        let directory = executable.parent().unwrap().to_str().unwrap();
        assert!(args.iter().any(|a| a.contains(&toml_string(directory))));
    }
}
