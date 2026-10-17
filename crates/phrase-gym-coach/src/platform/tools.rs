use std::io::Read;
use std::path::{Path, PathBuf};

use crate::adapter::CliAdapter;
use crate::adapter::environment::HostEnvironment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Discovery {
    Native(PathBuf),
    // npm packages and wrapper scripts start through an interpreter such as Node, which the
    // allowlisted child PATH does not contain, so they fail with exit 127 inside an attempt.
    Script(PathBuf),
    Missing,
}

// Finder and Start-menu launches do not run shell initialisation, so the documented install
// locations are checked before whatever PATH the GUI process happens to have.
pub fn candidates(
    adapter: CliAdapter,
    host: &HostEnvironment,
    system_directories: &[&Path],
) -> Vec<PathBuf> {
    let name = adapter.executable_name();
    let mut found = documented_locations(adapter, host);
    found.extend(
        system_directories
            .iter()
            .map(|directory| directory.join(file_name(name))),
    );
    if let Some(path) = host.get("PATH") {
        found.extend(std::env::split_paths(path).map(|directory| directory.join(file_name(name))));
    }
    found
}

pub fn discover(adapter: CliAdapter, host: &HostEnvironment) -> Discovery {
    let system: Vec<&Path> = SYSTEM_DIRECTORIES.iter().map(Path::new).collect();
    discover_in(adapter, host, &system)
}

// The first native executable wins even behind a script earlier in the search order; a script is
// reported only when no native install exists.
pub fn discover_in(
    adapter: CliAdapter,
    host: &HostEnvironment,
    system_directories: &[&Path],
) -> Discovery {
    let mut script = None;
    let resolved = candidates(adapter, host, system_directories)
        .into_iter()
        .filter(|candidate| candidate.is_absolute() && candidate.is_file())
        .filter_map(|candidate| candidate.canonicalize().ok());
    for executable in resolved {
        if !is_script(&executable) {
            return Discovery::Native(executable);
        }
        script.get_or_insert(executable);
    }
    script.map_or(Discovery::Missing, Discovery::Script)
}

fn is_script(path: &Path) -> bool {
    let mut start = [0_u8; 2];
    std::fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut start))
        .is_ok_and(|()| &start == b"#!")
}

#[cfg(not(windows))]
const SYSTEM_DIRECTORIES: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin"];
#[cfg(windows)]
const SYSTEM_DIRECTORIES: &[&str] = &[];

fn file_name(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

fn under(base: Option<&std::ffi::OsStr>, relative: &str) -> Option<PathBuf> {
    base.map(|base| Path::new(base).join(relative))
}

#[cfg(not(windows))]
fn documented_locations(adapter: CliAdapter, host: &HostEnvironment) -> Vec<PathBuf> {
    let name = adapter.executable_name();
    under(host.get("HOME"), &format!(".local/bin/{name}"))
        .into_iter()
        .collect()
}

#[cfg(windows)]
fn documented_locations(adapter: CliAdapter, host: &HostEnvironment) -> Vec<PathBuf> {
    let name = file_name(adapter.executable_name());
    under(host.get("USERPROFILE"), &format!(r".local\bin\{name}"))
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    const NATIVE: &[u8] = b"\xcf\xfa\xed\xfe native";
    const NODE_SCRIPT: &[u8] = b"#!/usr/bin/env node\nrequire('./cli.js');\n";

    fn install(path: &Path, content: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn home_and_path(root: &Path) -> (PathBuf, PathBuf, HostEnvironment) {
        let home = root.join("Ünï home");
        let on_path = root.join("path bin");
        let host = HostEnvironment::from_variables([
            (OsString::from("HOME"), home.clone().into_os_string()),
            (OsString::from("USERPROFILE"), home.clone().into_os_string()),
            (OsString::from("PATH"), on_path.clone().into_os_string()),
        ]);
        (home, on_path, host)
    }

    fn local_bin(home: &Path) -> PathBuf {
        if cfg!(windows) {
            home.join(r".local\bin")
        } else {
            home.join(".local/bin")
        }
    }

    #[test]
    fn prefers_documented_locations_and_resolves_symlinks_to_the_real_binary() {
        let root = tempfile::tempdir().unwrap();
        let (home, on_path, host) = home_and_path(root.path());
        assert_eq!(
            discover_in(CliAdapter::Claude, &host, &[]),
            Discovery::Missing
        );
        let path_copy = on_path.join(file_name("claude"));
        install(&path_copy, NATIVE);
        assert_eq!(
            discover_in(CliAdapter::Claude, &host, &[]),
            Discovery::Native(path_copy.canonicalize().unwrap())
        );
        let documented = local_bin(&home).join(file_name("claude"));
        install(&documented, NATIVE);
        assert_eq!(
            discover_in(CliAdapter::Claude, &host, &[]),
            Discovery::Native(documented.canonicalize().unwrap())
        );
        assert_eq!(
            discover_in(CliAdapter::Codex, &host, &[]),
            Discovery::Missing
        );
    }

    #[test]
    fn reports_node_script_installs_as_scripts_unless_a_native_build_exists() {
        let root = tempfile::tempdir().unwrap();
        let (home, on_path, host) = home_and_path(root.path());
        let script = local_bin(&home).join(file_name("codex"));
        install(&script, NODE_SCRIPT);
        assert_eq!(
            discover_in(CliAdapter::Codex, &host, &[]),
            Discovery::Script(script.canonicalize().unwrap())
        );
        let native = on_path.join(file_name("codex"));
        install(&native, NATIVE);
        assert_eq!(
            discover_in(CliAdapter::Codex, &host, &[]),
            Discovery::Native(native.canonicalize().unwrap())
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn ignores_the_legacy_claude_local_wrapper() {
        let root = tempfile::tempdir().unwrap();
        let (home, _, host) = home_and_path(root.path());
        install(&home.join(".claude/local/claude"), NATIVE);
        assert_eq!(
            discover_in(CliAdapter::Claude, &host, &[]),
            Discovery::Missing
        );
    }
}
