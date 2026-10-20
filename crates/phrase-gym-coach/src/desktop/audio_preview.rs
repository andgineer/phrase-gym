use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::platform::paths::CoachPaths;
use crate::transfer::pending;

#[cfg(target_os = "macos")]
pub fn play(path: &Path, paths: &CoachPaths, stop: &Arc<AtomicBool>) -> Result<(), String> {
    let bytes = crate::platform::files::read_selected(path, pending::MAX_AUDIO)?;
    let directory = crate::preparation::attempt::private_directory(paths)?;
    let audio = directory.join("selected-reference.wav");
    std::fs::write(&audio, bytes).map_err(|_| "Cannot stage the selected reference preview")?;
    let executable = std::path::PathBuf::from("/usr/bin/afplay");
    let report = crate::preparation::readiness::execute(
        crate::process::ProcessSpec {
            program: executable,
            args: vec![audio.into_os_string()],
            cwd: directory.clone(),
            env: crate::adapter::environment::child_environment(
                &crate::adapter::environment::HostEnvironment::current(),
                Path::new("/usr/bin"),
            ),
            stdout: directory.join("preview.stdout"),
            stderr: directory.join("preview.stderr"),
            limits: crate::process::Limits {
                output_bytes: 4096,
                deadline: std::time::Duration::from_secs(15),
                grace: std::time::Duration::from_secs(1),
            },
        },
        stop,
    )?;
    let _ = std::fs::remove_dir_all(directory);
    if report.ending == crate::process::Ending::Exited(Some(0)) {
        Ok(())
    } else {
        Err("Could not play the selected original. Check its WAV encoding and output device; no lesson was published by this preview".into())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn play(_path: &Path, _paths: &CoachPaths, _stop: &Arc<AtomicBool>) -> Result<(), String> {
    Err("Local audio preview is currently available on macOS; use browser reference playback after publication".into())
}
