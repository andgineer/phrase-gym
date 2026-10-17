use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

// Only what a CLI needs to find its own login and a locale; never provider keys, proxies or the
// parent CLI session variables a terminal or another agent may have exported.
#[cfg(unix)]
const INHERITED: &[&str] = &[
    "HOME", "USER", "LOGNAME", "SHELL", "TMPDIR", "LANG", "LC_ALL", "LC_CTYPE",
];
#[cfg(windows)]
const INHERITED: &[&str] = &[
    "SystemRoot",
    "SystemDrive",
    "windir",
    "ComSpec",
    "PATHEXT",
    "USERPROFILE",
    "HOMEDRIVE",
    "HOMEPATH",
    "APPDATA",
    "LOCALAPPDATA",
    "ProgramData",
    "ProgramFiles",
    "ProgramFiles(x86)",
    "ProgramW6432",
    "CommonProgramFiles",
    "TEMP",
    "TMP",
    "USERNAME",
    "USERDOMAIN",
    "COMPUTERNAME",
    "NUMBER_OF_PROCESSORS",
    "PROCESSOR_ARCHITECTURE",
    "OS",
];

#[derive(Clone, Debug, Default)]
pub struct HostEnvironment {
    variables: Vec<(OsString, OsString)>,
}

impl HostEnvironment {
    pub fn current() -> Self {
        Self::from_variables(std::env::vars_os())
    }

    pub fn from_variables(variables: impl IntoIterator<Item = (OsString, OsString)>) -> Self {
        Self {
            variables: variables.into_iter().collect(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&OsStr> {
        self.variables
            .iter()
            .find(|(key, _)| same_name(key, name))
            .map(|(_, value)| value.as_os_str())
    }
}

#[cfg(windows)]
fn same_name(key: &OsStr, name: &str) -> bool {
    key.to_str()
        .is_some_and(|key| key.eq_ignore_ascii_case(name))
}

#[cfg(not(windows))]
fn same_name(key: &OsStr, name: &str) -> bool {
    key == name
}

pub fn child_environment(
    host: &HostEnvironment,
    executable_directory: &Path,
) -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = INHERITED
        .iter()
        .filter_map(|name| {
            host.get(name)
                .map(|value| (OsString::from(name), value.to_os_string()))
        })
        .collect();
    let mut search = vec![executable_directory.to_path_buf()];
    search.extend(system_directories(host));
    if let Ok(path) = std::env::join_paths(search) {
        environment.push((OsString::from("PATH"), path));
    }
    environment
}

#[cfg(unix)]
fn system_directories(_host: &HostEnvironment) -> Vec<PathBuf> {
    ["/usr/bin", "/bin", "/usr/sbin", "/sbin"]
        .into_iter()
        .map(PathBuf::from)
        .collect()
}

#[cfg(windows)]
fn system_directories(host: &HostEnvironment) -> Vec<PathBuf> {
    let root = host
        .get("SystemRoot")
        .map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
    vec![
        root.join("System32"),
        root.clone(),
        root.join(r"System32\Wbem"),
        root.join(r"System32\WindowsPowerShell\v1.0"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(pairs: &[(&str, &str)]) -> HostEnvironment {
        HostEnvironment::from_variables(
            pairs
                .iter()
                .map(|(key, value)| (OsString::from(key), OsString::from(value))),
        )
    }

    fn names(environment: &[(OsString, OsString)]) -> Vec<String> {
        environment
            .iter()
            .map(|(key, _)| key.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn provider_keys_and_parent_agent_sessions_never_reach_the_child() {
        let leaked = [
            ("ANTHROPIC_API_KEY", "sk-ant"),
            ("ANTHROPIC_AUTH_TOKEN", "token"),
            ("ANTHROPIC_BASE_URL", "https://proxy.invalid"),
            ("CLAUDE_CODE_USE_BEDROCK", "1"),
            ("OPENAI_API_KEY", "sk-openai"),
            ("OPENAI_BASE_URL", "https://proxy.invalid"),
            ("CODEX_API_KEY", "sk-codex"),
            ("CLAUDE_CODE_SESSION_ID", "parent-session"),
            ("CLAUDECODE", "1"),
            ("HTTPS_PROXY", "http://proxy.invalid"),
            ("PATH", "/Users/someone/.npm/bin:/usr/bin"),
            ("PHRASE_GYM_TERMINAL_CANARY", "1"),
        ];
        let environment = child_environment(&host(&leaked), Path::new("/tools/bin"));
        let present = names(&environment);
        for (name, _) in leaked.iter().filter(|(name, _)| *name != "PATH") {
            assert!(!present.contains(&(*name).to_owned()), "{name} leaked");
        }
        let path = environment
            .iter()
            .find(|(key, _)| key == "PATH")
            .map(|(_, value)| value.to_string_lossy().into_owned())
            .unwrap();
        assert!(!path.contains(".npm"), "{path}");
        assert!(path.starts_with(&*Path::new("/tools/bin").to_string_lossy()));
    }

    #[cfg(unix)]
    #[test]
    fn keeps_only_the_login_locale_and_temporary_directory_variables() {
        let environment = child_environment(
            &host(&[
                ("HOME", "/Users/learner"),
                ("USER", "learner"),
                ("LANG", "sr_RS.UTF-8"),
                ("TMPDIR", "/var/tmp/x"),
                ("EDITOR", "vim"),
            ]),
            Path::new("/opt/codex/bin"),
        );
        assert_eq!(
            names(&environment),
            ["HOME", "USER", "TMPDIR", "LANG", "PATH"]
        );
        assert_eq!(
            environment.last().unwrap().1,
            OsString::from("/opt/codex/bin:/usr/bin:/bin:/usr/sbin:/sbin")
        );
    }

    #[cfg(windows)]
    #[test]
    fn matches_windows_variable_names_case_insensitively() {
        let environment = child_environment(
            &host(&[
                ("SYSTEMROOT", r"D:\Win"),
                ("UserProfile", r"C:\Users\Learner"),
            ]),
            Path::new(r"C:\Tools"),
        );
        assert_eq!(names(&environment), ["SystemRoot", "USERPROFILE", "PATH"]);
        let path = environment.last().unwrap().1.to_string_lossy().into_owned();
        assert!(
            path.starts_with(r"C:\Tools;D:\Win\System32;D:\Win"),
            "{path}"
        );
    }
}
