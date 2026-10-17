use std::path::{Path, PathBuf};

use crate::adapter::CliAdapter;
use crate::adapter::environment::HostEnvironment;

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

pub fn discover(adapter: CliAdapter, host: &HostEnvironment) -> Option<PathBuf> {
    let system: Vec<&Path> = SYSTEM_DIRECTORIES.iter().map(Path::new).collect();
    discover_in(adapter, host, &system)
}

pub fn discover_in(
    adapter: CliAdapter,
    host: &HostEnvironment,
    system_directories: &[&Path],
) -> Option<PathBuf> {
    candidates(adapter, host, system_directories)
        .into_iter()
        .filter(|candidate| candidate.is_absolute() && candidate.is_file())
        .find_map(|candidate| candidate.canonicalize().ok())
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
    let home = host.get("HOME");
    let name = adapter.executable_name();
    let mut locations = vec![under(home, &format!(".local/bin/{name}"))];
    if adapter == CliAdapter::Claude {
        locations.push(under(home, ".claude/local/claude"));
    }
    locations.into_iter().flatten().collect()
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

    fn executable(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"").unwrap();
    }

    #[test]
    fn prefers_documented_locations_and_resolves_symlinks_to_the_real_binary() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("Ünï home");
        let on_path = root.path().join("path bin");
        let local = if cfg!(windows) {
            home.join(r".local\bin")
        } else {
            home.join(".local/bin")
        };
        let host = HostEnvironment::from_variables([
            (OsString::from("HOME"), home.clone().into_os_string()),
            (OsString::from("USERPROFILE"), home.clone().into_os_string()),
            (OsString::from("PATH"), on_path.clone().into_os_string()),
        ]);
        assert_eq!(discover_in(CliAdapter::Claude, &host, &[]), None);
        let path_copy = on_path.join(file_name("claude"));
        executable(&path_copy);
        assert_eq!(
            discover_in(CliAdapter::Claude, &host, &[]),
            Some(path_copy.canonicalize().unwrap())
        );
        let documented = local.join(file_name("claude"));
        executable(&documented);
        assert_eq!(
            discover_in(CliAdapter::Claude, &host, &[]),
            Some(documented.canonicalize().unwrap())
        );
        assert_eq!(discover_in(CliAdapter::Codex, &host, &[]), None);
    }
}
