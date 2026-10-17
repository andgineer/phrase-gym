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
    use super::super::CliAdapter;
    use super::super::environment::HostEnvironment;
    use super::super::invocation::{AttemptFiles, Effort, ModelSelection};
    use super::*;

    #[cfg(unix)]
    const EXECUTABLE: &str = "/opt/homebrew/Caskroom/codex/0.159.2 Ünï/bin/codex";
    #[cfg(unix)]
    const INSTALL_READ: &str = r#""/opt/homebrew/Caskroom/codex/0.159.2 Ünï/bin"="read""#;
    #[cfg(unix)]
    const ATTEMPT: &str = "/Users/learner/Library/Caches/Coach Ünï/attempts/attempt Ђ";
    #[cfg(unix)]
    const BOOTSTRAP: &str = "Read your task from \
        /Users/learner/Library/Caches/Coach Ünï/attempts/attempt Ђ/prompt.md and execute it.";
    #[cfg(unix)]
    const CODEX_HOME: &str = "/Users/learner/Library/Application Support/Coach Ünï/codex-home";
    #[cfg(windows)]
    const EXECUTABLE: &str = r"C:\Users\learner\AppData\Local\Programs\Codex Ünï\codex.exe";
    #[cfg(windows)]
    const INSTALL_READ: &str =
        r#""C:\\Users\\learner\\AppData\\Local\\Programs\\Codex Ünï"="read""#;
    #[cfg(windows)]
    const ATTEMPT: &str = r"C:\Users\learner\AppData\Local\Coach Ünï\attempts\attempt Ђ";
    #[cfg(windows)]
    const BOOTSTRAP: &str = r"Read your task from C:\Users\learner\AppData\Local\Coach Ünï\attempts\attempt Ђ\prompt.md and execute it.";
    #[cfg(windows)]
    const CODEX_HOME: &str = r"C:\Users\learner\AppData\Local\Coach Ünï\data\codex-home";

    fn host() -> HostEnvironment {
        HostEnvironment::from_variables(
            [
                ("HOME", "/Users/learner"),
                ("USERPROFILE", r"C:\Users\learner"),
                ("OPENAI_API_KEY", "sk-openai"),
                ("CODEX_HOME", "/Users/learner/.codex"),
            ]
            .map(|(key, value)| (OsString::from(key), OsString::from(value))),
        )
    }

    fn build(executable: &Path, effort: Option<Effort>) -> Invocation {
        let attempt = AttemptFiles::new(ATTEMPT.into(), "prompt.md").unwrap();
        let model = ModelSelection::new("gpt-5.6-luna", effort).unwrap();
        CliAdapter::Codex
            .invocation(&LaunchContext {
                executable,
                attempt: &attempt,
                model: &model,
                host: &host(),
                codex_home: Path::new(CODEX_HOME),
            })
            .unwrap()
    }

    fn expected_arguments(effort: &[&str]) -> Vec<String> {
        let profile = [
            r#"permissions.coach_job={filesystem={":root"="deny", ":minimal"="read", "#,
            r#"":slash_tmp"="deny", ":tmpdir"="deny", "#,
            INSTALL_READ,
            r#", ":workspace_roots"={"."="write"}}, network={enabled=false}}"#,
        ]
        .concat();
        let mut expected: Vec<String> = [
            "exec",
            "--skip-git-repo-check",
            "--ignore-user-config",
            "--ignore-rules",
            "--ephemeral",
            "-c",
            r#"default_permissions="coach_job""#,
            "-c",
            &profile,
            "-c",
            r#"forced_login_method="chatgpt""#,
            "-c",
            r#"approval_policy="never""#,
            "-c",
            r#"web_search="disabled""#,
            "-c",
            r#"history.persistence="none""#,
            "-c",
            "check_for_update_on_startup=false",
            "--disable",
            "apps",
            "--disable",
            "browser_use",
            "--disable",
            "browser_use_external",
            "--disable",
            "computer_use",
            "--disable",
            "goals",
            "--disable",
            "hooks",
            "--disable",
            "image_generation",
            "--disable",
            "in_app_browser",
            "--disable",
            "multi_agent",
            "--disable",
            "plugins",
            "--disable",
            "remote_plugin",
            "--disable",
            "skill_mcp_dependency_install",
            "--disable",
            "tool_suggest",
            "--disable",
            "view_image",
            "--model",
            "gpt-5.6-luna",
        ]
        .map(str::to_owned)
        .to_vec();
        expected.extend(effort.iter().map(|value| (*value).to_owned()));
        expected.push(BOOTSTRAP.to_owned());
        expected
    }

    #[test]
    fn passes_exactly_the_verified_arguments_with_low_effort() {
        let invocation = build(Path::new(EXECUTABLE), Some(Effort::Low));
        assert_eq!(
            invocation.arguments(),
            expected_arguments(&["-c", "model_reasoning_effort=low"])
        );
        assert_eq!(invocation.program, Path::new(EXECUTABLE));
        assert_eq!(invocation.cwd, Path::new(ATTEMPT));
    }

    #[test]
    fn passes_exactly_the_verified_arguments_without_effort() {
        let invocation = build(Path::new(EXECUTABLE), None);
        assert_eq!(invocation.arguments(), expected_arguments(&[]));
    }

    #[test]
    fn adds_only_the_coach_codex_home_to_the_allowlisted_environment() {
        let env = build(Path::new(EXECUTABLE), Some(Effort::Low)).env;
        let mut expected = child_environment(&host(), Path::new(EXECUTABLE).parent().unwrap());
        expected.push((OsString::from("CODEX_HOME"), OsString::from(CODEX_HOME)));
        assert_eq!(env, expected);
        assert!(!env.iter().any(|(key, _)| key == "OPENAI_API_KEY"));
    }

    #[test]
    fn quotes_install_paths_as_toml_strings() {
        assert_eq!(
            toml_string(r"C:\Program Files\Codex"),
            r#""C:\\Program Files\\Codex""#
        );
        assert_eq!(toml_string("a\"b\nc"), r#""a\"b\u000Ac""#);
        let executable = std::env::temp_dir().join("Ünï \"tools\"").join("codex");
        let args = build(&executable, Some(Effort::Low)).arguments();
        let directory = executable.parent().unwrap().to_str().unwrap();
        assert!(args.iter().any(|a| a.contains(&toml_string(directory))));
    }
}
