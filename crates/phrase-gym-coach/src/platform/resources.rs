use std::path::{Path, PathBuf};

// macOS bundles keep resources in Contents/Resources beside Contents/MacOS; Windows and portable
// builds keep them in a `resources` folder beside the executable.
pub fn resource_directory(executable: &Path) -> PathBuf {
    let directory = executable.parent().unwrap_or(executable);
    let contents = directory.parent();
    let in_bundle = directory.file_name().is_some_and(|name| name == "MacOS")
        && contents
            .and_then(Path::file_name)
            .is_some_and(|name| name == "Contents");
    match contents {
        Some(contents) if in_bundle => contents.join("Resources"),
        _ => directory.join("resources"),
    }
}

// Development builds have no resource folder; there the helper sits beside the executable.
pub fn locate(executable: &Path, file_name: &str) -> Option<PathBuf> {
    let directory = executable.parent()?;
    [resource_directory(executable), directory.to_path_buf()]
        .into_iter()
        .map(|candidate| candidate.join(file_name))
        .find(|candidate| candidate.is_file())
}

pub fn executable_name(stem: &str) -> String {
    format!("{stem}{}", std::env::consts::EXE_SUFFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"").unwrap();
    }

    #[test]
    fn finds_resources_inside_a_macos_bundle_with_unicode_and_spaces() {
        let root = tempfile::tempdir().unwrap();
        let app = root
            .path()
            .join("Tëst Ђ folder/Phrase Gym Coach.app/Contents");
        let executable = app.join("MacOS/phrase-gym-coach");
        touch(&executable);
        touch(&app.join("Resources/helper"));
        assert_eq!(resource_directory(&executable), app.join("Resources"));
        assert_eq!(
            locate(&executable, "helper"),
            Some(app.join("Resources/helper"))
        );
    }

    #[test]
    fn finds_resources_beside_a_portable_executable_and_in_development() {
        let root = tempfile::tempdir().unwrap();
        let executable = root.path().join("Coach Ünï").join("phrase-gym-coach.exe");
        touch(&executable);
        assert_eq!(
            resource_directory(&executable),
            root.path().join("Coach Ünï").join("resources")
        );
        assert_eq!(locate(&executable, "helper.exe"), None);
        touch(&root.path().join("Coach Ünï").join("helper.exe"));
        assert_eq!(
            locate(&executable, "helper.exe"),
            Some(root.path().join("Coach Ünï").join("helper.exe"))
        );
        touch(&root.path().join("Coach Ünï/resources/helper.exe"));
        assert_eq!(
            locate(&executable, "helper.exe"),
            Some(root.path().join("Coach Ünï/resources/helper.exe"))
        );
    }
}
